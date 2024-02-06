use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use std::env;

#[get("/healthcheck")]
async fn healthcheck() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/")]
async fn hello() -> impl Responder {
    "MultiplayerBase Authentication Server"
}

const HOST: &'static str = "0.0.0.0";
const PORT: u16 = 8000;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .service(healthcheck)
            .service(hello)
    })
    .bind((HOST, PORT))?
    .run()
    .await
}
