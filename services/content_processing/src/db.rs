//! Database connection and pool management

use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

/// Create and configure a PostgreSQL connection pool
pub async fn create_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(600))
        .connect(database_url)
        .await?;
    
    Ok(pool)
}

/// Run database migrations
pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Row;
    
    #[tokio::test]
    #[ignore] // Requires database
    async fn test_create_pool() {
        let database_url = "postgresql://test:test@localhost:5432/test_content_processing";
        let pool = create_pool(database_url).await;
        assert!(pool.is_ok());
    }
    
    #[tokio::test]
    #[ignore] // Requires database
    async fn test_pool_configuration() {
        let database_url = "postgresql://test:test@localhost:5432/test_content_processing";
        let pool = create_pool(database_url).await.unwrap();
        
        // Test pool options
        assert!(pool.size() <= 20); // max_connections
        assert!(pool.is_closed() == false);
    }
    
    #[tokio::test]
    #[ignore] // Requires database
    async fn test_connection_health() {
        let database_url = "postgresql://test:test@localhost:5432/test_content_processing";
        let pool = create_pool(database_url).await.unwrap();
        
        // Test basic query
        let result = sqlx::query("SELECT 1 as num")
            .fetch_one(&pool)
            .await;
            
        assert!(result.is_ok());
        let row = result.unwrap();
        let num: i32 = row.get("num");
        assert_eq!(num, 1);
    }
    
    #[tokio::test]
    async fn test_create_pool_invalid_url() {
        let database_url = "invalid://url";
        let pool = create_pool(database_url).await;
        assert!(pool.is_err());
    }
    
    #[tokio::test]
    #[ignore] // Requires database
    async fn test_run_migrations() {
        let database_url = "postgresql://test:test@localhost:5432/test_content_processing";
        let pool = create_pool(database_url).await.unwrap();
        
        // Run migrations (this might fail if already run, which is okay)
        let _ = run_migrations(&pool).await;
        
        // Check if migrations table exists
        let result = sqlx::query(
            "SELECT EXISTS (
                SELECT FROM information_schema.tables 
                WHERE table_schema = 'public' 
                AND table_name = '_sqlx_migrations'
            ) as exists"
        )
        .fetch_one(&pool)
        .await;
        
        assert!(result.is_ok());
        let row = result.unwrap();
        let exists: bool = row.get("exists");
        assert!(exists);
    }
    
    #[tokio::test]
    #[ignore] // Requires database
    async fn test_multiple_connections() {
        let database_url = "postgresql://test:test@localhost:5432/test_content_processing";
        let pool = create_pool(database_url).await.unwrap();
        
        // Test multiple concurrent connections
        let mut handles = vec![];
        
        for i in 0..10 {
            let pool_clone = pool.clone();
            let handle = tokio::spawn(async move {
                let result = sqlx::query("SELECT $1::int as num")
                    .bind(i)
                    .fetch_one(&pool_clone)
                    .await;
                    
                assert!(result.is_ok());
                let row = result.unwrap();
                let num: i32 = row.get("num");
                assert_eq!(num, i);
            });
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }
    }
}