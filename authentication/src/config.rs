use crate::auth;
use actix_cors::Cors;
use chrono::Duration;
use std::env;

pub fn get_db_url() -> String {
    let url = env::var("POSTGRES_URL");
    if let Ok(url) = url {
        return url;
    }

    let password_text = env::var("POSTGRES_PASSWORD");
    let password_file = env::var("POSTGRES_PASSWORD_FILE");
    let password = if let Ok(password_text) = password_text {
        password_text
    } else {
        let password_file = password_file
            .expect("Expected either POSTGRES_PASSWORD or POSTGRES_PASSWORD_FILE to be set");

        use std::fs;
        fs::read_to_string(password_file)
            .expect("The file at POSTGRES_PASSWORD_FILE should contain the database password")
    };

    let user = env::var("POSTGRES_USER").unwrap_or("postgres".to_string());
    let db = env::var("POSTGRES_DB").unwrap_or("postgres".to_string());
    let host = env::var("POSTGRES_HOST").unwrap_or("localhost".to_string());
    let port = env::var("POSTGRES_PORT").unwrap_or("5432".to_string());

    format!("postgres://{user}:{password}@{host}:{port}/{db}")
}

pub fn get_identity_config() -> auth::identity::IdentityConfig {
    let secret_text = env::var("IDENTITY_SECRET");
    let secret_file = env::var("IDENTITY_SECRET_FILE");

    let secret = if let Ok(secret_text) = secret_text {
        secret_text
    } else {
        let secret_file =
            secret_file.expect("Expected either IDENTITY_SECRET or IDENTITY_SECRET_FILE to be set");

        use std::fs;
        fs::read_to_string(secret_file)
            .expect("The file at IDENTITY_SECRET_FILE should contain the identity signing secret")
    };

    let expires_in_seconds = env::var("IDENTITY_EXPIRES_IN_SECS")
        .ok()
        .and_then(|p| p.parse::<i64>().ok())
        .unwrap_or(3600);
    let expires_in = Duration::seconds(expires_in_seconds);

    let refresh_secret_text = env::var("REFRESH_SECRET");
    let refresh_secret_file = env::var("REFRESH_SECRET_FILE");

    let refresh_secret = if let Ok(refresh_secret_text) = refresh_secret_text {
        refresh_secret_text
    } else {
        let refresh_secret_file = refresh_secret_file
            .expect("Expected either REFRESH_SECRET or REFRESH_SECRET_FILE to be set");

        use std::fs;
        fs::read_to_string(refresh_secret_file).expect(
            "The file at REFRESH_SECRET_FILE should contain the refresh token signing secret",
        )
    };

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
