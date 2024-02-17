use crate::auth::identity::IdentityConfig;
use actix_cors::Cors;
use chrono::Duration;
use std::env;

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

fn get_db_url() -> String {
    let url = get_secret_text_or_file("POSTGRES_URL");
    if let Some(url) = url {
        return url;
    }

    let password = get_required_secret_text_or_file("POSTGRES_PASSWORD");
    let user = get_secret_text_or_file("POSTGRES_USER").unwrap_or("postgres".to_string());
    let db = get_secret_text_or_file("POSTGRES_DB").unwrap_or("postgres".to_string());
    let host = get_secret_text_or_file("POSTGRES_HOST").unwrap_or("localhost".to_string());
    let port = get_secret_text_or_file("POSTGRES_PORT").unwrap_or("5432".to_string());

    format!("postgres://{user}:{password}@{host}:{port}/{db}")
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

pub struct OAuthClientSecrets {
    pub id: String,
    pub secret: String,
}

fn get_oauth_client_secrets() -> OAuthClientSecrets {
    OAuthClientSecrets {
        id: get_required_secret_text_or_file("OAUTH_CLIENT_ID"),
        secret: get_required_secret_text_or_file("OAUTH_CLIENT_SECRET"),
    }
}

pub struct SteamConfig {
    pub app_id: String,
    pub web_api_key: String,
}

fn get_steam_config() -> SteamConfig {
    SteamConfig {
        app_id: get_required_secret_text_or_file("STEAM_APP_ID"),
        web_api_key: get_required_secret_text_or_file("STEAM_WEB_API_KEY"),
    }
}

lazy_static::lazy_static! {
    pub static ref DB_URL: String = get_db_url();
    pub static ref IDENTITY_CONFIG: IdentityConfig = get_identity_config();
    pub static ref OAUTH_CLIENT_SECRETS: OAuthClientSecrets = get_oauth_client_secrets();
    pub static ref STEAM_CONFIG: SteamConfig = get_steam_config();
}

pub fn get_cors_config() -> Cors {
    let cors = Cors::permissive().supports_credentials();

    if let Some(origins) = get_secret_text_or_file("ALLOWED_ORIGINS") {
        origins
            .split(",")
            .fold(cors, |cors, origin| cors.allowed_origin(origin))
    } else {
        cors
    }
}
