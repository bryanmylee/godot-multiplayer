use serde::Deserialize;

use crate::config::{get_oauth_client_secrets, OAuthClientSecrets};

const OAUTH_TOKEN_URI: &'static str = "https://oauth2.googleapis.com/token";

lazy_static::lazy_static! {
    static ref OAUTH_CLIENT_SECRETS: OAuthClientSecrets = get_oauth_client_secrets();
}

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
        let resp = client.post(OAUTH_TOKEN_URI).form(&params).send().await?;

        let token: TokenPayload = resp.json().await?;

        Ok(token.access_token)
    }
}

pub struct RealPlayGamesExchangeAuthCodeService;

impl PlayGamesExchangeAuthCodeService for RealPlayGamesExchangeAuthCodeService {}
