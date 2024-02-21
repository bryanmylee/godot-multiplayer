use crate::config::OAUTH_CLIENT_SECRETS;
use serde::Deserialize;
use std::time::Duration;

const OAUTH_TOKEN_URL: &'static str = "https://oauth2.googleapis.com/token";

#[derive(Deserialize)]
#[allow(dead_code)]
struct TokenPayload {
    access_token: String,
    expires_in: i64,
    scope: String,
    token_type: String,
}

#[async_trait::async_trait]
pub trait PlayGamesExchangeAuthCodeService: Sync {
    async fn get_access_token(
        &self,
        auth_code: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let params = [
            ("client_id", &OAUTH_CLIENT_SECRETS.id),
            ("client_secret", &OAUTH_CLIENT_SECRETS.secret),
            ("code", &auth_code.to_string()),
            ("grant_type", &"authorization_code".to_string()),
        ];

        let client = reqwest::Client::new();
        let resp = client
            .post(OAUTH_TOKEN_URL)
            .timeout(Duration::from_secs(5))
            .form(&params)
            .send()
            .await?;

        let token: TokenPayload = resp.json().await?;

        Ok(token.access_token)
    }
}

pub struct RealPlayGamesExchangeAuthCodeService;

impl PlayGamesExchangeAuthCodeService for RealPlayGamesExchangeAuthCodeService {}
