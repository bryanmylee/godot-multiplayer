use bb8::CustomizeConnection;
use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncConnection, AsyncPgConnection};

/// Short-hand for the database pool type to use throughout the app.
pub type DbPool = Pool<AsyncPgConnection>;
pub type DbConnection = AsyncPgConnection;
pub type DbError = Box<dyn std::error::Error + Send + Sync>;

pub async fn initialize_db_pool(db_url: &str) -> Pool<AsyncPgConnection> {
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(db_url);
    let mut pool_builder = Pool::builder();
    if cfg!(test) {
        pool_builder = pool_builder
            .max_size(1)
            .connection_customizer(Box::new(TestConnection));
    }
    pool_builder
        .build(config)
        .await
        .expect("The database URL should be a valid Postgres connection string")
}

#[derive(Debug)]
struct TestConnection;

#[async_trait::async_trait]
impl<C, E> CustomizeConnection<C, E> for TestConnection
where
    C: AsyncConnection + 'static,
    E: 'static,
{
    async fn on_acquire(&self, conn: &mut C) -> Result<(), E> {
        conn.begin_test_transaction()
            .await
            .expect("Failed to start test transaction");
        Ok(())
    }
}
