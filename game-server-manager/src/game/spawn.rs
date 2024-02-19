use crate::game::{Game, GameDescription, GamesData};
use crate::ServiceKey;
use actix_web::{error, post, web, HttpResponse};
use chrono::{DateTime, Utc};
use std::fs::File;
use std::time::SystemTime;
use subprocess::{Popen, PopenConfig, Redirection};

#[post("/spawn/")]
async fn spawn(
    service_key: ServiceKey,
    games_data: web::Data<GamesData>,
) -> actix_web::Result<HttpResponse> {
    service_key.validate()?;

    let mut games = games_data
        .games
        .write()
        .expect("Failed to get write lock on games");

    let Some((game_port, game_value)) = games.get_available_entry() else {
        return Err(error::ErrorServiceUnavailable(
            "No available game ports remaining",
        ));
    };

    let now = SystemTime::now();
    let now: DateTime<Utc> = now.into();
    let now_str = now.to_rfc3339();

    let log_filename = format!("{game_port}-{now_str}.log");
    let log_stdout =
        File::create(&log_filename).expect(&format!("Could not open log file {log_filename}"));
    let log_stderr =
        File::create(&log_filename).expect(&format!("Could not open log file {log_filename}"));

    let config = PopenConfig {
        stdout: Redirection::File(log_stdout),
        stderr: Redirection::File(log_stderr),
        detached: true,
        ..Default::default()
    };

    let process = Popen::create(
        &[
            "/game-server/run",
            "--server",
            "--headless",
            &format!("--port={game_port}"),
        ],
        config,
    )
    .map_err(error::ErrorInternalServerError)?;

    let game = Game {
        process,
        created_at: now,
        port: game_port,
    };

    let game_description: GameDescription = (&game).into();

    _ = std::mem::replace(game_value, Some(game));

    Ok(HttpResponse::Created().json(game_description))
}
