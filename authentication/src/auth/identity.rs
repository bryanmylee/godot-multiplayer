use crate::user::User;
use actix_web::{error, http, web, FromRequest, HttpMessage};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct IdentityConfig {
    pub secret: String,
    pub expires_in: Duration,
    pub refresh_secret: String,
    pub refresh_expires_in: Duration,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct IdentityClaims {
    pub sub: Uuid,
    pub iat: u64,
    pub exp: u64,
}

impl IdentityClaims {
    pub fn encode(&self, config: &IdentityConfig) -> String {
        jsonwebtoken::encode(
            &Header::default(),
            self,
            &EncodingKey::from_secret(config.secret.as_ref()),
        )
        .expect("Failed to encode access token")
    }

    pub fn decode(config: &IdentityConfig, token: &str) -> Result<Self, error::Error> {
        match jsonwebtoken::decode::<Self>(
            token,
            &DecodingKey::from_secret(config.secret.as_ref()),
            &Validation::default(),
        ) {
            Ok(payload) => Ok(payload.claims),
            Err(err) => Err(error::ErrorBadRequest(err)),
        }
    }
}

/// A user's verified identity provided by their server access token.
#[derive(Debug, Clone)]
pub struct Identity {
    pub user_id: Uuid,
}

impl Identity {
    pub fn from_user_id(user_id: &Uuid) -> Self {
        Identity {
            user_id: user_id.clone(),
        }
    }

    pub fn from_user(user: &User) -> Self {
        Identity {
            user_id: user.id.clone(),
        }
    }

    pub fn generate_token(&self, config: &IdentityConfig) -> String {
        let now = Utc::now();
        let iat = now.timestamp() as u64;
        let exp = (now + config.expires_in).timestamp() as u64;
        let claims = IdentityClaims {
            sub: self.user_id,
            iat,
            exp,
        };
        claims.encode(config)
    }
}

impl FromRequest for Identity {
    type Error = error::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let Some(token) = req
            .headers()
            .get(http::header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .map(|h| h.split_at(7).1.to_string())
        else {
            return ready(Err(error::ErrorUnauthorized("Missing Bearer token")));
        };

        let config = req
            .app_data::<web::Data<IdentityConfig>>()
            .expect("IdentityConfig is available in app data");

        let claims = match IdentityClaims::decode(config, &token) {
            Ok(claims) => claims,
            Err(err) => return ready(Err(error::ErrorBadRequest(err))),
        };

        let identity = Identity::from_user_id(&claims.sub);
        req.extensions_mut().insert::<Identity>(identity.clone());

        ready(Ok(identity))
    }
}
