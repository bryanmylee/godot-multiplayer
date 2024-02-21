use actix::Actor;
use actix_web::{get, middleware, web, App, HttpServer, Responder};
use matchmaking::{config, queue, websocket};

#[get("/")]
async fn hello() -> impl Responder {
    "MultiplayerBase Matchmaking"
}

const HOST: &'static str = "127.0.0.1";
const PORT: u16 = 8100;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    let identity_config = config::IDENTITY_CONFIG.clone();
    let websocket_server = web::Data::new(websocket::server::WebsocketServer::new().start());
    let queue_data = web::Data::new(queue::QueueData::new());

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::NormalizePath::new(
                middleware::TrailingSlash::Always,
            ))
            .wrap(config::get_cors_config())
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(identity_config.clone()))
            .app_data(websocket_server.clone())
            .app_data(queue_data.clone())
            .service(hello)
            .service(websocket::listen)
    })
    .bind((HOST, PORT))?
    .run()
    .await
}
