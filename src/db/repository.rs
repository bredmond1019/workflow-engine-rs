use diesel::RunQueryDsl;
use diesel::Table;
use diesel::query_builder::InsertStatement;
use diesel::query_dsl::methods::ExecuteDsl;
use diesel::{PgConnection, sql_query};
use std::sync::Arc;

use super::session::DbPool;

pub struct Model<T, R>
where
    T: Table + Copy,
    R: diesel::Insertable<T> + Clone,
{
    table: T,
    records: R,
}

pub struct Repository<T, R>
where
    T: Table + Copy,
    R: diesel::Insertable<T> + Clone,
{
    db_pool: Arc<DbPool>,
    model: Model<T, R>,
}

impl<T, R> Repository<T, R>
where
    T: Table + Copy,
    R: diesel::Insertable<T> + Clone,
{
    pub fn new(db_pool: Arc<DbPool>, model: Model<T, R>) -> Self {
        Self { db_pool, model }
    }

    pub fn create_record(&self) -> Result<(), diesel::result::Error> {
        let mut conn = self
            .db_pool
            .get()
            .expect("Failed to get connection from pool");

        diesel::insert_into(self.model.table)
            .values(&self.model.records)
            .execute(&mut conn)?;

        Ok(())
    }
}

pub fn clear_all_tables(conn: &mut PgConnection) -> Result<(), diesel::result::Error> {
    sql_query("TRUNCATE TABLE articles, collections CASCADE").execute(conn)?;
    Ok(())
}
