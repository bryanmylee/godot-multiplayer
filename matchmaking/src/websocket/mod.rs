pub mod server;
pub mod session;

use self::{server::WebsocketServer, session::WebsocketSession};
use crate::identity::{BearerToken, Identity, IdentityConfig};
use actix::Addr;
use actix_web::{error, get, web, HttpRequest, Responder};
use actix_web_actors::ws;
use serde::Deserialize;

#[derive(Deserialize)]
struct ListenParams {
    token: String,
}

#[get("/listen/")]
async fn listen(
    params: web::Query<ListenParams>,
    req: HttpRequest,
    stream: web::Payload,
    server_address: web::Data<Addr<WebsocketServer>>,
    id_config: web::Data<IdentityConfig>,
) -> actix_web::Result<impl Responder> {
    let token = BearerToken::new(&params.token);
    let identity = Identity::from_token(&id_config, &token)?;

    ws::start(
        WebsocketSession::new(identity.user_id, server_address.as_ref().clone()),
        &req,
        stream,
    )
    .map_err(error::ErrorInternalServerError)
}
