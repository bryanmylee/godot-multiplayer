use actix_web::{get, middleware, web, App, HttpServer, Responder};
use authentication::{user, DbPool};
use diesel::{r2d2, PgConnection};
use std::env;

#[get("/")]
async fn hello() -> impl Responder {
    "MultiplayerBase Authentication Server"
}

const HOST: &'static str = "0.0.0.0";
const PORT: u16 = 8000;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    let pool = initialize_db_pool();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .wrap(middleware::NormalizePath::new(
                middleware::TrailingSlash::Always,
            ))
            .service(hello)
            .service(web::scope("/user").configure(user::config_service))
    })
    .bind((HOST, PORT))?
    .run()
    .await
}

/// Initialize a database connection pool based on the `DATABASE_URL` environment variable.
///
/// See more: <https://docs.rs/diesel/latest/diesel/r2d2/index.html>.
fn initialize_db_pool() -> DbPool {
    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let manager = r2d2::ConnectionManager::<PgConnection>::new(conn_spec);
    r2d2::Pool::builder()
        .build(manager)
        .expect("DATABASE_URL should be a valid Postgres connection string")
}
