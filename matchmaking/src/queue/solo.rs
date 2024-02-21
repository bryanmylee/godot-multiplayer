use crate::{identity::Identity, queue::QueueData};
use actix_web::{post, web, HttpResponse, Responder};

pub fn config_service(cfg: &mut web::ServiceConfig) {
    cfg.service(join).service(leave);
}

#[post("/join/")]
async fn join(
    identity: Identity,
    queued_data: web::Data<QueueData>,
) -> actix_web::Result<impl Responder> {
    let mut solo_queue = queued_data
        .solo
        .write()
        .expect("Failed to get write lock on solo queue");

    solo_queue.insert_player(identity.user_id);

    Ok(HttpResponse::Ok().finish())
}

#[post("/leave/")]
async fn leave(
    identity: Identity,
    queued_data: web::Data<QueueData>,
) -> actix_web::Result<impl Responder> {
    let mut solo_queue = queued_data
        .solo
        .write()
        .expect("Failed to get write lock on solo queue");

    solo_queue.remove_player(&identity.user_id);

    Ok(HttpResponse::Ok().finish())
}
