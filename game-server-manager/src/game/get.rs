use crate::game::{GameDescription, GamesData};
use crate::ServiceKey;
use actix_web::{get, web, HttpResponse};

#[get("/list/")]
async fn list(
    service_key: ServiceKey,
    games_data: web::Data<GamesData>,
) -> actix_web::Result<HttpResponse> {
    service_key.validate()?;

    let games = games_data
        .games
        .read()
        .expect("Failed to get read lock on games");

    Ok(HttpResponse::Ok().json(games.get_all_active_description()))
}

#[get("/port/{port}/")]
async fn find_by_port(
    port: web::Path<u16>,
    service_key: ServiceKey,
    games_data: web::Data<GamesData>,
) -> actix_web::Result<HttpResponse> {
    service_key.validate()?;

    let games = games_data
        .games
        .read()
        .expect("Failed to get read lock on games");

    let Some(game) = games.find_by_port(port.into_inner()) else {
        return Ok(HttpResponse::NotFound().finish());
    };

    let game: GameDescription = game.into();
    Ok(HttpResponse::Ok().json(game))
}
