use diesel::{r2d2, PgConnection};

pub mod schema;
pub mod user;

/// Short-hand for the database pool type to use throughout the app.
pub type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;

pub type DbError = Box<dyn std::error::Error + Send + Sync>;
