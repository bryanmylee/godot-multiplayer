use crate::game::GamesData;
use crate::ServiceKey;
use actix_web::{get, web, HttpResponse};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct GameServerInfo {
    id: String,
    name: String,
    internal_host: String,
    internal_port: u16,
    external_port: u16,
}

#[derive(Debug, Clone, Serialize)]
struct SpawnResponse {
    process_id: u32,
    port: u16,
}

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
