use actix_web::{get, middleware, web, App, HttpServer, Responder};
use game_server_manager::game;

#[get("/")]
async fn hello() -> impl Responder {
    "MultiplayerBase Game Server Manager"
}

const HOST: &'static str = "0.0.0.0";
const PORT: u16 = 8500;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::NormalizePath::new(
                middleware::TrailingSlash::Always,
            ))
            .wrap(middleware::Logger::default())
            .service(hello)
            .service(web::scope("/game").configure(game::config_service))
    })
    .bind((HOST, PORT))?
    .run()
    .await
}
