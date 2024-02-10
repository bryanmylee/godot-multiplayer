use actix_web::{get, middleware, web, App, HttpServer, Responder};
use authentication::{auth, config, user};
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

    let pool = config::initialize_db_pool(&config::get_db_url());
    let identity_config = config::get_identity_config();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(identity_config.clone()))
            .wrap(middleware::NormalizePath::new(
                middleware::TrailingSlash::Always,
            ))
            .wrap(config::get_cors_config())
            .wrap(middleware::Logger::default())
            .service(hello)
            .service(web::scope("/auth").configure(auth::config_service))
            .service(web::scope("/user").configure(user::config_service))
    })
    .bind((HOST, PORT))?
    .run()
    .await
}
