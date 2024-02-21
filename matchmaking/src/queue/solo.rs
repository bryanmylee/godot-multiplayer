use crate::{
    identity::{BearerToken, IdentityService},
    queue::QueueData,
    websocket::{server::WebsocketServer, RoutingToServerMessage},
};
use actix::Addr;
use actix_web::{error, post, web, HttpResponse, Responder};

pub fn config_service(cfg: &mut web::ServiceConfig) {
    cfg.service(join).service(leave);
}

#[post("/join/")]
async fn join(
    token: BearerToken,
    server_address: web::Data<Addr<WebsocketServer>>,
    queue_data: web::Data<QueueData>,
    id_service: web::Data<dyn IdentityService>,
) -> actix_web::Result<impl Responder> {
    let identity = id_service.get_identity(&token)?;

    let mut solo_queue = queue_data
        .solo
        .write()
        .expect("Failed to get write lock on solo queue");

    let player = solo_queue.insert_user(identity.user_id)?;

    server_address
        .as_ref()
        .clone()
        .try_send(RoutingToServerMessage::CheckQueue)
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(player))
}

#[post("/leave/")]
async fn leave(
    token: BearerToken,
    id_service: web::Data<dyn IdentityService>,
    queue_data: web::Data<QueueData>,
) -> actix_web::Result<impl Responder> {
    let identity = id_service.get_identity(&token)?;

    let mut solo_queue = queue_data
        .solo
        .write()
        .expect("Failed to get write lock on solo queue");

    let player = solo_queue.remove_player(&identity.user_id)?;

    Ok(HttpResponse::Ok().json(player))
}

#[cfg(test)]
mod tests {
    use crate::{
        identity::{Identity, IdentityService},
        queue,
    };
    use actix_web::{http::header::AUTHORIZATION, test, App};
    use std::sync::Arc;
    use uuid::Uuid;

    use super::*;

    #[actix_web::test]
    async fn server_sends_ready_message_to_clients_when_ready() {
        struct RandomIdService;

        impl IdentityService for RandomIdService {
            fn get_identity(&self, _: &BearerToken) -> Result<Identity, actix_web::error::Error> {
                Ok(Identity {
                    user_id: Uuid::new_v4(),
                })
            }
        }

        let id_service = web::Data::from(Arc::new(RandomIdService) as Arc<dyn IdentityService>);
        let queue_data = web::Data::new(queue::QueueData::new());

        let app = test::init_service(
            App::new()
                .app_data(id_service)
                .app_data(queue_data)
                .service(join),
        )
        .await;

        let req = test::TestRequest::post()
            .insert_header((AUTHORIZATION, "Bearer 0000"))
            .uri("/join/")
            .to_request();
        test::call_service(&app, req).await;

        let req = test::TestRequest::post()
            .insert_header((AUTHORIZATION, "Bearer 0000"))
            .uri("/join/")
            .to_request();
        let resp = test::call_service(&app, req).await;

        println!("{:?}", test::read_body(resp).await);
    }
}
