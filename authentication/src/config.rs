use crate::auth;
use actix_cors::Cors;
use chrono::Duration;
use std::env;

fn get_secret_text_or_file(var: &str) -> String {
    let secret_text = env::var(var);
    let secret_file = env::var(format!("{var}_FILE"));

    if let Ok(secret_text) = secret_text {
        secret_text
    } else {
        let secret_file =
            secret_file.expect(format!("Expected either {var} or {var}_FILE to be set").as_str());

        use std::fs;
        fs::read_to_string(secret_file)
            .expect(format!("The file at {var}_FILE should contain the secret").as_str())
    }
}

pub fn get_db_url() -> String {
    let url = env::var("POSTGRES_URL");
    if let Ok(url) = url {
        return url;
    }

    let password = get_secret_text_or_file("POSTGRES_PASSWORD");
    let user = env::var("POSTGRES_USER").unwrap_or("postgres".to_string());
    let db = env::var("POSTGRES_DB").unwrap_or("postgres".to_string());
    let host = env::var("POSTGRES_HOST").unwrap_or("localhost".to_string());
    let port = env::var("POSTGRES_PORT").unwrap_or("5432".to_string());

    format!("postgres://{user}:{password}@{host}:{port}/{db}")
}

pub fn get_identity_config() -> auth::identity::IdentityConfig {
    let secret = get_secret_text_or_file("IDENTITY_SECRET");
    let expires_in_seconds = env::var("IDENTITY_EXPIRES_IN_SECS")
        .ok()
        .and_then(|p| p.parse::<i64>().ok())
        .unwrap_or(3600);
    let expires_in = Duration::seconds(expires_in_seconds);

    let refresh_secret = get_secret_text_or_file("REFRESH_SECRET");
    let refresh_expires_in_days = env::var("REFRESH_EXPIRES_IN_DAYS")
        .ok()
        .and_then(|p| p.parse::<i64>().ok())
        .unwrap_or(7);
    let refresh_expires_in = Duration::days(refresh_expires_in_days);

    auth::identity::IdentityConfig {
        secret,
        expires_in,
        refresh_secret,
        refresh_expires_in,
    }
}

pub fn get_cors_config() -> Cors {
    let cors = Cors::permissive().supports_credentials();

    if let Ok(origins) = env::var("ALLOWED_ORIGINS") {
        origins
            .split(",")
            .fold(cors, |cors, origin| cors.allowed_origin(origin))
    } else {
        cors
    }
}

pub struct OAuthClientSecrets {
    pub id: String,
    pub secret: String,
}

pub fn get_oauth_client_secrets() -> OAuthClientSecrets {
    let id = get_secret_text_or_file("OAUTH_CLIENT_ID");
    let secret = get_secret_text_or_file("OAUTH_CLIENT_SECRET");
    OAuthClientSecrets { id, secret }
}
