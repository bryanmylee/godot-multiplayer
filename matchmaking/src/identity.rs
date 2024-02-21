use actix_web::{error, http, FromRequest};
use chrono::Duration;
use jsonwebtoken::{DecodingKey, Validation};
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

    pub fn from_token(
        id_config: &IdentityConfig,
        token: &BearerToken,
    ) -> Result<Self, error::Error> {
        let claims = match IdentityClaims::decode(id_config, &token.0) {
            Ok(claims) => claims,
            Err(err) => return Err(error::ErrorUnauthorized(err)),
        };

        Ok(Identity::from_user_id(&claims.sub))
    }
}

pub struct BearerToken(String);

impl BearerToken {
    pub fn new(token: &str) -> BearerToken {
        BearerToken(token.to_string())
    }
}

impl FromRequest for BearerToken {
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

        let token = BearerToken(token);

        ready(Ok(token))
    }
}

pub trait IdentityService: Sync {
    fn get_identity(&self, token: &BearerToken) -> Result<Identity, error::Error>;
}

pub struct RealIdentityService {
    id_config: IdentityConfig,
}

impl RealIdentityService {
    pub fn new(id_config: IdentityConfig) -> RealIdentityService {
        RealIdentityService { id_config }
    }
}

impl IdentityService for RealIdentityService {
    fn get_identity(&self, token: &BearerToken) -> Result<Identity, error::Error> {
        Identity::from_token(&self.id_config, token)
    }
}
