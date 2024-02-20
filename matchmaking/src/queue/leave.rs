use crate::identity::Identity;
use crate::player::PlayersData;
use actix_web::{post, web, HttpResponse};

#[post("/leave/")]
async fn leave(
    identity: Identity,
    players_data: web::Data<PlayersData>,
) -> actix_web::Result<HttpResponse> {
    let mut queued = players_data
        .queued
        .write()
        .expect("Failed to get write lock on player queue");

    queued.retain(|p| p.user_id != identity.user_id);

    Ok(HttpResponse::Ok().finish())
}
