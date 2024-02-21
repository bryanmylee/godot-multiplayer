use std::sync::Arc;

use actix::Actor;
use actix_web::{get, middleware, web, App, HttpServer, Responder};
use matchmaking::{
    config::{self, MATCHMAKING_CONFIG},
    identity::{IdentityService, RealIdentityService},
    queue, websocket,
};

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

    let queue_data = web::Data::new(queue::QueueData::new());
    let server =
        websocket::server::WebsocketServer::new(queue_data.clone(), MATCHMAKING_CONFIG.clone());
    let server_address = web::Data::new(server.start());

    HttpServer::new(move || {
        let id_service = web::Data::from(Arc::new(RealIdentityService::new(
            config::IDENTITY_CONFIG.clone(),
        )) as Arc<dyn IdentityService>);

        App::new()
            .wrap(middleware::NormalizePath::new(
                middleware::TrailingSlash::Always,
            ))
            .wrap(config::get_cors_config())
            .wrap(middleware::Logger::default())
            .app_data(queue_data.clone())
            .app_data(server_address.clone())
            .app_data(id_service)
            .service(hello)
            .service(websocket::listen)
            .service(web::scope("/queue").configure(queue::config_service))
    })
    .bind((HOST, PORT))?
    .run()
    .await
}
