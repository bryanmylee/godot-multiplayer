use crate::game::GamesData;
use crate::ServiceKey;
use actix_web::{error, post, web, HttpResponse};

#[post("/kill/{port}/")]
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

    let game = games.find_mut_by_port(port.into_inner());

    match game {
        Some(game) => game
            .process
            .terminate()
            .map_err(error::ErrorInternalServerError)?,
        None => return Ok(HttpResponse::NotFound().finish()),
    };

    _ = std::mem::replace(game, None);

    Ok(HttpResponse::Ok().finish())
}
