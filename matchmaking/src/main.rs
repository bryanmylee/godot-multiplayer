use actix_web::{get, middleware, web, App, HttpServer, Responder};
use matchmaking::{config, player, start};

#[get("/")]
async fn hello() -> impl Responder {
    "MultiplayerBase Matchmaking"
}

const HOST: &'static str = "0.0.0.0";
const PORT: u16 = 8100;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    let identity_config = config::IDENTITY_CONFIG.clone();
    let queued_players_data = web::Data::new(player::PlayersData::new());

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::NormalizePath::new(
                middleware::TrailingSlash::Always,
            ))
            .wrap(config::get_cors_config())
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(identity_config.clone()))
            .app_data(queued_players_data.clone())
            .service(hello)
            .service(start)
    })
    .bind((HOST, PORT))?
    .run()
    .await
}
