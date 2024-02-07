use std::{
    future::{ready, Ready},
    time::Duration,
};

use actix_web::{error, http, web, FromRequest, HttpMessage};
use jsonwebtoken::{DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod oauth2;

pub fn config_service(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/oauth2").configure(oauth2::config_service));
}

#[derive(Clone, Debug)]
pub struct JwtConfig {
    pub secret: String,
    pub expires_in: Duration,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Provider {
    OAuth2,
    Steam,
    GooglePlayGames,
    AppleGameCenter,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Claims {
    pub sub: String,
    pub iat: u64,
    pub exp: u64,
}

pub struct JwtMiddleware {
    pub user_id: Uuid,
}

impl FromRequest for JwtMiddleware {
    type Error = error::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let token = req
            .headers()
            .get(http::header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .map(|h| h.split_at(7).1.to_string());
        let Some(token) = token else {
            return ready(Err(error::ErrorUnauthorized("Missing Bearer token")));
        };

        let jwt_config = req
            .app_data::<web::Data<JwtConfig>>()
            .expect("JwtConfig is available in app data");

        let Ok(payload) = jsonwebtoken::decode::<Claims>(
            &token,
            &DecodingKey::from_secret(jwt_config.secret.as_ref()),
            &Validation::default(),
        ) else {
            return ready(Err(error::ErrorUnauthorized("Invalid Bearer token")));
        };
        
        let user_id = Uuid::parse_str(&payload.claims.sub).expect("sub claim to be a UUID");
        req.extensions_mut()
            .insert::<Uuid>(user_id.to_owned());
        
        ready(Ok(JwtMiddleware { user_id }))
    }
}
