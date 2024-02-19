use crate::player::PlayersData;
use crate::{identity::Identity, player::Player};
use actix_web::{post, web, HttpResponse};

#[post("/join/")]
async fn join(
    identity: Identity,
    players_data: web::Data<PlayersData>,
) -> actix_web::Result<HttpResponse> {
    let mut queued = players_data
        .queued
        .write()
        .expect("Failed to get write lock on player queue");

    let player = Player {
        user_id: identity.user_id,
    };

    queued.push(player);

    Ok(HttpResponse::Ok().finish())
}
