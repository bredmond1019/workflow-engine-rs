// File: src/db/tenant.rs
//
// Multi-tenancy implementation with row-level security

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::pg::PgConnection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::core::error::WorkflowError;
use super::schema::tenants;

/// Tenant information
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = tenants)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub database_schema: String,
    pub isolation_mode: String,
    pub settings: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New tenant creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTenant {
    pub name: String,
    pub isolation_mode: TenantIsolationMode,
    pub settings: Option<serde_json::Value>,
}

/// Tenant isolation modes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TenantIsolationMode {
    /// Complete database separation (separate schemas)
    Schema,
    /// Row-level security with tenant_id filtering
    RowLevel,
    /// Hybrid approach with both schema and row-level
    Hybrid,
}

impl TenantIsolationMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            TenantIsolationMode::Schema => "schema",
            TenantIsolationMode::RowLevel => "row_level",
            TenantIsolationMode::Hybrid => "hybrid",
        }
    }
}

/// Tenant context for database operations
#[derive(Debug, Clone)]
pub struct TenantContext {
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub database_schema: String,
    pub isolation_mode: TenantIsolationMode,
}

/// Tenant-aware database connection
pub struct TenantConnection {
    conn: PooledConnection<ConnectionManager<PgConnection>>,
    context: TenantContext,
}

impl TenantConnection {
    /// Create a new tenant-aware connection
    pub fn new(
        conn: PooledConnection<ConnectionManager<PgConnection>>,
        context: TenantContext,
    ) -> Self {
        Self { conn, context }
    }

    /// Get the underlying connection
    pub fn conn(&mut self) -> &mut PgConnection {
        &mut self.conn
    }

    /// Execute a query with tenant context
    pub fn with_tenant_context<F, R>(&mut self, f: F) -> Result<R, WorkflowError>
    where
        F: FnOnce(&mut PgConnection, &TenantContext) -> Result<R, diesel::result::Error>,
    {
        // Set the search path for schema isolation
        if matches!(self.context.isolation_mode, TenantIsolationMode::Schema | TenantIsolationMode::Hybrid) {
            let schema_query = format!("SET search_path TO {}, public", self.context.database_schema);
            diesel::sql_query(schema_query)
                .execute(&mut self.conn)
                .map_err(|e| WorkflowError::DatabaseError { message: e.to_string() })?;
        }

        // Set row-level security context
        if matches!(self.context.isolation_mode, TenantIsolationMode::RowLevel | TenantIsolationMode::Hybrid) {
            let rls_query = format!(
                "SET LOCAL app.current_tenant_id = '{}'",
                self.context.tenant_id
            );
            diesel::sql_query(rls_query)
                .execute(&mut self.conn)
                .map_err(|e| WorkflowError::DatabaseError { message: e.to_string() })?;
        }

        f(&mut self.conn, &self.context)
            .map_err(|e| WorkflowError::DatabaseError { message: e.to_string() })
    }
}

/// Tenant management service
pub struct TenantManager {
    pool: Arc<Pool<ConnectionManager<PgConnection>>>,
    tenant_cache: Arc<RwLock<HashMap<Uuid, Tenant>>>,
}

impl TenantManager {
    pub fn new(pool: Arc<Pool<ConnectionManager<PgConnection>>>) -> Self {
        Self {
            pool,
            tenant_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new tenant
    pub async fn create_tenant(&self, new_tenant: NewTenant) -> Result<Tenant, WorkflowError> {
        let pool = Arc::clone(&self.pool);
        let tenant_id = Uuid::new_v4();
        let schema_name = format!("tenant_{}", tenant_id.to_string().replace("-", "_"));
        
        let tenant = tokio::task::spawn_blocking(move || -> Result<Tenant, WorkflowError> {
            let mut conn = pool.get()
                .map_err(|e| WorkflowError::DatabaseError { message: e.to_string() })?;
            
            conn.transaction(|conn| {
                // Create tenant record
                let now = Utc::now();
                let tenant = Tenant {
                    id: tenant_id,
                    name: new_tenant.name,
                    database_schema: schema_name.clone(),
                    isolation_mode: new_tenant.isolation_mode.as_str().to_string(),
                    settings: new_tenant.settings.unwrap_or_else(|| serde_json::json!({})),
                    is_active: true,
                    created_at: now,
                    updated_at: now,
                };
                
                diesel::insert_into(tenants::table)
                    .values(&tenant)
                    .execute(conn)?;
                
                // Create schema if using schema isolation
                if matches!(new_tenant.isolation_mode, TenantIsolationMode::Schema | TenantIsolationMode::Hybrid) {
                    diesel::sql_query(format!("CREATE SCHEMA IF NOT EXISTS {}", schema_name))
                        .execute(conn)?;
                    
                    // Clone base tables to tenant schema
                    Self::clone_schema_tables(conn, &schema_name)?;
                }
                
                // Set up row-level security if needed
                if matches!(new_tenant.isolation_mode, TenantIsolationMode::RowLevel | TenantIsolationMode::Hybrid) {
                    Self::setup_row_level_security(conn, tenant_id)?;
                }
                
                Ok(tenant)
            })
        }).await
            .map_err(|e| WorkflowError::RuntimeError { message: e.to_string() })??;
        
        // Cache the tenant
        self.tenant_cache.write().await.insert(tenant.id, tenant.clone());
        
        Ok(tenant)
    }

    /// Get a tenant by ID
    pub async fn get_tenant(&self, tenant_id: Uuid) -> Result<Tenant, WorkflowError> {
        // Check cache first
        if let Some(tenant) = self.tenant_cache.read().await.get(&tenant_id) {
            return Ok(tenant.clone());
        }
        
        // Load from database
        let pool = Arc::clone(&self.pool);
        let tenant = tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()
                .map_err(|e| WorkflowError::DatabaseError { message: e.to_string() })?;
            
            tenants::table
                .find(tenant_id)
                .first::<Tenant>(&mut conn)
                .map_err(|e| WorkflowError::DatabaseError { message: e.to_string() })
        }).await
            .map_err(|e| WorkflowError::RuntimeError { message: e.to_string() })??;
        
        // Update cache
        self.tenant_cache.write().await.insert(tenant.id, tenant.clone());
        
        Ok(tenant)
    }

    /// Get a tenant-aware connection
    pub async fn get_tenant_connection(
        &self,
        tenant_id: Uuid,
    ) -> Result<TenantConnection, WorkflowError> {
        let tenant = self.get_tenant(tenant_id).await?;
        let conn = self.pool.get()
            .map_err(|e| WorkflowError::DatabaseError { message: e.to_string() })?;
        
        let isolation_mode = match tenant.isolation_mode.as_str() {
            "schema" => TenantIsolationMode::Schema,
            "row_level" => TenantIsolationMode::RowLevel,
            "hybrid" => TenantIsolationMode::Hybrid,
            _ => return Err(WorkflowError::ValidationError { message:
                format!("Invalid isolation mode: {}", tenant.isolation_mode)
            }),
        };
        
        let context = TenantContext {
            tenant_id: tenant.id,
            tenant_name: tenant.name,
            database_schema: tenant.database_schema,
            isolation_mode,
        };
        
        Ok(TenantConnection::new(conn, context))
    }

    /// Clone schema tables for tenant isolation
    fn clone_schema_tables(conn: &mut PgConnection, schema_name: &str) -> Result<(), diesel::result::Error> {
        // List of tables to clone (excluding system tables)
        let tables = vec![
            "events", "event_store", "event_snapshots", 
            "event_subscriptions", "event_projections",
            "agents", "users"
        ];
        
        for table in tables {
            // Create table in tenant schema with same structure
            let create_query = format!(
                "CREATE TABLE {}.{} (LIKE public.{} INCLUDING ALL)",
                schema_name, table, table
            );
            diesel::sql_query(create_query).execute(conn)?;
        }
        
        Ok(())
    }

    /// Set up row-level security for a tenant
    fn setup_row_level_security(conn: &mut PgConnection, tenant_id: Uuid) -> Result<(), diesel::result::Error> {
        // Add tenant_id column to tables if not exists
        let tables = vec![
            "events", "event_store", "event_snapshots", 
            "event_subscriptions", "event_projections",
            "agents", "users"
        ];
        
        for table in &tables {
            // Add tenant_id column
            let alter_query = format!(
                "ALTER TABLE {} ADD COLUMN IF NOT EXISTS tenant_id UUID DEFAULT '{}'",
                table, tenant_id
            );
            diesel::sql_query(alter_query).execute(conn)?;
            
            // Create index on tenant_id
            let index_query = format!(
                "CREATE INDEX IF NOT EXISTS idx_{}_tenant_id ON {} (tenant_id)",
                table, table
            );
            diesel::sql_query(index_query).execute(conn)?;
        }
        
        // Enable row-level security
        for table in &tables {
            let enable_rls = format!("ALTER TABLE {} ENABLE ROW LEVEL SECURITY", table);
            diesel::sql_query(enable_rls).execute(conn)?;
            
            // Create policy for tenant isolation
            let policy_name = format!("{}_tenant_isolation", table);
            let create_policy = format!(
                "CREATE POLICY IF NOT EXISTS {} ON {} 
                 FOR ALL 
                 USING (tenant_id = current_setting('app.current_tenant_id')::uuid)",
                policy_name, table
            );
            diesel::sql_query(create_policy).execute(conn)?;
        }
        
        Ok(())
    }

    /// Update tenant settings
    pub async fn update_tenant_settings(
        &self,
        tenant_id: Uuid,
        settings: serde_json::Value,
    ) -> Result<(), WorkflowError> {
        let pool = Arc::clone(&self.pool);
        
        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()
                .map_err(|e| WorkflowError::DatabaseError { message: e.to_string() })?;
            
            diesel::update(tenants::table.find(tenant_id))
                .set((
                    tenants::settings.eq(settings),
                    tenants::updated_at.eq(Utc::now()),
                ))
                .execute(&mut conn)
                .map_err(|e| WorkflowError::DatabaseError { message: e.to_string() })?;
            
            Ok(())
        }).await
            .map_err(|e| WorkflowError::RuntimeError { message: e.to_string() })?
    }

    /// Deactivate a tenant
    pub async fn deactivate_tenant(&self, tenant_id: Uuid) -> Result<(), WorkflowError> {
        let pool = Arc::clone(&self.pool);
        
        tokio::task::spawn_blocking(move || -> Result<(), WorkflowError> {
            let mut conn = pool.get()
                .map_err(|e| WorkflowError::DatabaseError { message: e.to_string() })?;
            
            diesel::update(tenants::table.find(tenant_id))
                .set((
                    tenants::is_active.eq(false),
                    tenants::updated_at.eq(Utc::now()),
                ))
                .execute(&mut conn)
                .map_err(|e| WorkflowError::DatabaseError { message: e.to_string() })?;
            
            Ok(())
        }).await
            .map_err(|e| WorkflowError::RuntimeError { message: e.to_string() })?;
        
        // Remove from cache
        self.tenant_cache.write().await.remove(&tenant_id);
        
        Ok(())
    }

    /// List all active tenants
    pub async fn list_active_tenants(&self) -> Result<Vec<Tenant>, WorkflowError> {
        let pool = Arc::clone(&self.pool);
        
        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()
                .map_err(|e| WorkflowError::DatabaseError { message: e.to_string() })?;
            
            tenants::table
                .filter(tenants::is_active.eq(true))
                .order(tenants::created_at.asc())
                .load::<Tenant>(&mut conn)
                .map_err(|e| WorkflowError::DatabaseError { message: e.to_string() })
        }).await
            .map_err(|e| WorkflowError::RuntimeError { message: e.to_string() })?
    }
}

/// Trait for tenant-aware repositories
#[async_trait]
pub trait TenantAware {
    /// Get the tenant context for this repository
    fn tenant_context(&self) -> &TenantContext;
    
    /// Ensure tenant isolation in queries
    fn apply_tenant_filter<'a>(&self, query: &'a str) -> String {
        match &self.tenant_context().isolation_mode {
            TenantIsolationMode::RowLevel | TenantIsolationMode::Hybrid => {
                format!("{} AND tenant_id = '{}'", query, self.tenant_context().tenant_id)
            }
            TenantIsolationMode::Schema => query.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_isolation_mode_conversion() {
        assert_eq!(TenantIsolationMode::Schema.as_str(), "schema");
        assert_eq!(TenantIsolationMode::RowLevel.as_str(), "row_level");
        assert_eq!(TenantIsolationMode::Hybrid.as_str(), "hybrid");
    }

    #[test]
    fn test_new_tenant_creation() {
        let new_tenant = NewTenant {
            name: "Test Company".to_string(),
            isolation_mode: TenantIsolationMode::RowLevel,
            settings: Some(serde_json::json!({
                "max_users": 100,
                "features": ["advanced_analytics"]
            })),
        };

        assert_eq!(new_tenant.name, "Test Company");
        assert_eq!(new_tenant.isolation_mode, TenantIsolationMode::RowLevel);
        assert!(new_tenant.settings.is_some());
    }
}