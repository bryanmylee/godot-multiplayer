pub mod config;
pub mod identity;
pub mod player;
pub mod websocket;

use actix::Addr;
use actix_web::{error, get, web, HttpRequest, Responder};
use actix_web_actors::ws;
use identity::{Identity, IdentityConfig};
use serde::Deserialize;
use websocket::{server::WebsocketServer, session::WebsocketSession};

#[derive(Deserialize)]
struct StartParams {
    token: String,
}

#[get("/start/")]
async fn start(
    params: web::Query<StartParams>,
    req: HttpRequest,
    stream: web::Payload,
    server_address: web::Data<Addr<WebsocketServer>>,
    id_config: web::Data<IdentityConfig>,
) -> actix_web::Result<impl Responder> {
    let identity = Identity::from_token(&id_config, &params.token)?;

    ws::start(
        WebsocketSession::new(identity.user_id, server_address.as_ref().clone()),
        &req,
        stream,
    )
    .map_err(error::ErrorInternalServerError)
}
