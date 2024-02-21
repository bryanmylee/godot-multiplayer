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

    let player = solo_queue.join_queue(identity.user_id)?;

    Ok(HttpResponse::Ok().json(player))
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

    let player = solo_queue.leave_queue(&identity.user_id)?;

    Ok(HttpResponse::Ok().json(player))
}
