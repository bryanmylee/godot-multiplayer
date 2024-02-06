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
    let db_url = get_db_url();
    let manager = r2d2::ConnectionManager::<PgConnection>::new(db_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("The database URL should be a valid Postgres connection string")
}

fn get_db_url() -> String {
    let url = std::env::var("POSTGRES_URL");
    if let Ok(url) = url {
        return url;
    }

    let password_text = std::env::var("POSTGRES_PASSWORD");
    let password_file = std::env::var("POSTGRES_PASSWORD_FILE");
    let password = if let Ok(password_text) = password_text {
        password_text
    } else {
        let Ok(password_file) = password_file else {
            panic!("Expected either POSTGRES_PASSWORD or POSTGRES_PASSWORD_FILE to be set");
        };

        use std::fs;
        fs::read_to_string(password_file)
            .expect("The file at POSTGRES_PASSWORD_FILE should contain the database password")
    };

    let user = std::env::var("POSTGRES_USER").unwrap_or("postgres".to_string());
    let db = std::env::var("POSTGRES_DB").unwrap_or("postgres".to_string());
    let host = std::env::var("POSTGRES_HOST").unwrap_or("localhost".to_string());
    let port = std::env::var("POSTGRES_PORT").unwrap_or("5432".to_string());

    format!("postgres://{user}:{password}@{host}:{port}/{db}")
}
