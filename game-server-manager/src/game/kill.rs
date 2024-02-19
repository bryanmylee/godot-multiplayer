use crate::game::GamesData;
use crate::ServiceKey;
use actix_web::{error, post, web, HttpResponse};

#[post("/kill/{port}")]
async fn kill(
    port: web::Path<u16>,
    service_key: ServiceKey,
    games_data: web::Data<GamesData>,
) -> actix_web::Result<HttpResponse> {
    service_key.validate()?;

    let mut games = games_data
        .games
        .write()
        .expect("Failed to get write lock on games");

    let Some(game) = games.find_mut_by_port(port.into_inner()) else {
        return Ok(HttpResponse::NotFound().finish());
    };

    game.process
        .terminate()
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().finish())
}
