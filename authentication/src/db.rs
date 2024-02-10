use crate::config::get_db_url;
use actix_web::error;
use diesel::{r2d2, Connection, PgConnection};
use log;

/// Short-hand for the database pool type to use throughout the app.
pub type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<r2d2::ConnectionManager<PgConnection>>;
pub type DbError = Box<dyn std::error::Error + Send + Sync>;

lazy_static::lazy_static! {
    static ref DB_POOL: DbPool = {
        let db_url = get_db_url();
        initialize_db_pool(&db_url)
    };
}

pub fn initialize_db_pool(db_url: &str) -> DbPool {
    let manager = r2d2::ConnectionManager::<PgConnection>::new(db_url);
    r2d2::Pool::builder()
        .max_size(match cfg!(test) {
            true => 1,
            false => 10,
        })
        .build(manager)
        .expect("The database URL should be a valid Postgres connection string")
}

/// If the database is started in a test configuration, a test transaction is
/// started to rollback any changes to the database after the test completes.
pub fn init() {
    log::info!("Initializing database");
    lazy_static::initialize(&DB_POOL);
    let mut conn = get_connection().expect("Failed to get a database connection");
    if cfg!(test) {
        conn.begin_test_transaction()
            .expect("Failed to start a test transaction");
    }
}

pub fn get_pool() -> DbPool {
    DB_POOL.to_owned()
}

pub fn get_connection() -> Result<DbConnection, error::Error> {
    DB_POOL.get().map_err(error::ErrorInternalServerError)
}
