use diesel::{
    PgConnection,
    r2d2::{self, ConnectionManager, Pool},
};

use std::env;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn init_pool() -> DbPool {
    let database_url: String = get_database_url();
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}

fn get_database_url() -> String {
    env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}
