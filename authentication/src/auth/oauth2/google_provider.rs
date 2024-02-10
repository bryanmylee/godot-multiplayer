use crate::auth::provider::{AuthProviderChangeset, AuthProviderInsert, AuthProviderType};
use crate::user::{User, UserInsert};
use actix_web::error;
use reqwest::StatusCode;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct GoogleUserInfo {
    pub id: String,
    pub email: Option<String>,
    pub verified_email: bool,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
    pub locale: Option<String>,
}

impl From<&GoogleUserInfo> for AuthProviderChangeset {
    fn from(value: &GoogleUserInfo) -> Self {
        AuthProviderChangeset {
            order: 0,
            email: value.email.clone(),
            email_verified: value.verified_email,
            display_name: value.name.clone(),
            user_name: value.email.clone(),
            picture_url: value.picture.clone(),
            locale: value.locale.clone(),
        }
    }
}

impl From<&GoogleUserInfo> for UserInsert {
    fn from(value: &GoogleUserInfo) -> Self {
        UserInsert {
            name: value.name.clone(),
        }
    }
}

impl GoogleUserInfo {
    pub fn into_provider_insert(self: &Self, user: &User) -> AuthProviderInsert {
        AuthProviderInsert {
            user_id: user.id,
            order: 0,
            provider_type: AuthProviderType::OAuth2,
            provider_id: self.id.clone(),
            email: self.email.clone(),
            email_verified: self.verified_email,
            display_name: self.name.clone(),
            user_name: self.email.clone(),
            picture_url: self.picture.clone(),
            locale: self.locale.clone(),
        }
    }
}

#[async_trait::async_trait]
pub trait GoogleUserInfoService {
    async fn get_info(self: &Self, token: &str) -> Result<GoogleUserInfo, error::Error>;
}

const USER_INFO_REQUEST_URI: &'static str = "https://www.googleapis.com/userinfo/v2/me";

pub struct RealGoogleUserInfoService;

#[async_trait::async_trait]
impl GoogleUserInfoService for RealGoogleUserInfoService {
    async fn get_info(&self, token: &str) -> Result<GoogleUserInfo, error::Error> {
        let client = reqwest::Client::new();
        let resp = client
            .get(USER_INFO_REQUEST_URI)
            .bearer_auth(token)
            .send()
            .await
            .map_err(error::ErrorInternalServerError)?;

        match resp.status() {
            StatusCode::OK => resp
                .json::<GoogleUserInfo>()
                .await
                .map_err(error::ErrorInternalServerError),
            StatusCode::UNAUTHORIZED => Err(error::ErrorUnauthorized(resp.status())),
            _ => Err(error::ErrorInternalServerError(
                "Failed to get user information",
            )),
        }
    }
}
