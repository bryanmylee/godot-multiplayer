pub mod server;
pub mod session;

use self::{server::WebsocketServer, session::WebsocketSession};
use crate::identity::{BearerToken, IdentityService};
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
    id_service: web::Data<dyn IdentityService>,
) -> actix_web::Result<impl Responder> {
    let token = BearerToken::new(&params.token);
    let identity = id_service.get_identity(&token)?;

    ws::start(
        WebsocketSession::new(identity.user_id, server_address.as_ref().clone()),
        &req,
        stream,
    )
    .map_err(error::ErrorInternalServerError)
}
