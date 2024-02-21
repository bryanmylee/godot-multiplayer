pub mod config;
pub mod identity;
pub mod player;
pub mod session;

use actix_web::{error, get, web, HttpRequest, Responder};
use actix_web_actors::ws;
use identity::Identity;

#[get("/start/")]
async fn start(
    identity: Identity,
    req: HttpRequest,
    stream: web::Payload,
) -> actix_web::Result<impl Responder> {
    println!("starting session");
    ws::start(session::WsSession::new(identity.user_id), &req, stream)
        .map_err(error::ErrorInternalServerError)?;
    Ok("")
}
