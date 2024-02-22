use crate::identity::IdentityConfig;
use actix_cors::Cors;
use chrono::Duration;
use std::env;

#[allow(dead_code)]
fn get_secret_text_or_file(var: &str) -> Option<String> {
    let secret_text = env::var(var);

    if let Ok(secret_text) = secret_text {
        Some(secret_text)
    } else {
        use std::fs;
        let Ok(secret_file) = env::var(format!("{var}_FILE")) else {
            return None;
        };
        fs::read_to_string(secret_file).ok()
    }
}

fn get_required_secret_text_or_file(var: &str) -> String {
    let secret_text = env::var(var);

    if let Ok(secret_text) = secret_text {
        secret_text
    } else {
        use std::fs;
        let secret_file = env::var(format!("{var}_FILE"))
            .expect(format!("Expected either {var} or {var}_FILE to be set").as_str());
        fs::read_to_string(secret_file)
            .expect(format!("The file at {var}_FILE should contain the secret").as_str())
    }
}

fn get_identity_config() -> IdentityConfig {
    let secret = get_required_secret_text_or_file("IDENTITY_SECRET");
    let expires_in_seconds = get_secret_text_or_file("IDENTITY_EXPIRES_IN_SECS")
        .and_then(|p| p.parse::<i64>().ok())
        .unwrap_or(3600);
    let expires_in = Duration::seconds(expires_in_seconds);

    let refresh_secret = get_required_secret_text_or_file("REFRESH_SECRET");
    let refresh_expires_in_days = get_secret_text_or_file("REFRESH_EXPIRES_IN_DAYS")
        .and_then(|p| p.parse::<i64>().ok())
        .unwrap_or(7);
    let refresh_expires_in = Duration::days(refresh_expires_in_days);

    IdentityConfig {
        secret,
        expires_in,
        refresh_secret,
        refresh_expires_in,
    }
}

#[derive(Debug, Clone)]
pub struct GameServerManagerConfig {
    pub game_server_external_host: String,
    pub url: String,
    pub service_key: String,
}

fn get_game_server_manager_config() -> GameServerManagerConfig {
    GameServerManagerConfig {
        game_server_external_host: get_required_secret_text_or_file("GAME_SERVER_EXTERNAL_HOST"),
        url: get_required_secret_text_or_file("GAME_SERVER_MANAGER_URL"),
        service_key: get_required_secret_text_or_file("GAME_SERVER_MANAGER_SERVICE_KEY"),
    }
}

#[derive(Debug, Clone)]
pub struct MatchmakingConfig {
    pub solo_game_min_size: u8,
    pub solo_game_desired_size: u8,
    pub solo_game_desired_max_wait_time: Duration,
}

impl Default for MatchmakingConfig {
    fn default() -> Self {
        MatchmakingConfig {
            solo_game_min_size: 2,
            solo_game_desired_size: 4,
            solo_game_desired_max_wait_time: Duration::minutes(1),
        }
    }
}

fn get_matchmaking_config() -> MatchmakingConfig {
    MatchmakingConfig {
        solo_game_min_size: 2,
        solo_game_desired_size: 4,
        solo_game_desired_max_wait_time: Duration::minutes(1),
    }
}

lazy_static::lazy_static! {
    pub static ref POSTGRES_URL: String = get_required_secret_text_or_file("POSTGRES_URL");
    pub static ref IDENTITY_CONFIG: IdentityConfig = get_identity_config();
    pub static ref GAME_SERVER_MANAGER_CONFIG: GameServerManagerConfig = get_game_server_manager_config();
    pub static ref MATCHMAKING_CONFIG: MatchmakingConfig = get_matchmaking_config();
}

pub fn get_cors_config() -> Cors {
    let cors = Cors::permissive().supports_credentials().allow_any_method();

    if let Some(origins) = get_secret_text_or_file("ALLOWED_ORIGINS") {
        origins
            .split(",")
            .fold(cors, |cors, origin| cors.allowed_origin(origin))
    } else {
        cors
    }
}
