use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use std::env;
use thiserror::Error;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database URL not found: {0}")]
    MissingDatabaseUrl(#[from] env::VarError),
    #[error("Failed to create database connection pool: {0}")]
    PoolCreationError(String),
}

pub fn init_pool() -> Result<DbPool, DatabaseError> {
    let database_url = get_database_url()?;
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .build(manager)
        .map_err(|e| DatabaseError::PoolCreationError(format!("Pool creation failed: {}", e)))?;
    Ok(pool)
}

fn get_database_url() -> Result<String, DatabaseError> {
    env::var("DATABASE_URL").map_err(DatabaseError::MissingDatabaseUrl)
}
