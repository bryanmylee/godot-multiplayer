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
}

impl IdentityConfig {
    pub fn generate_identity(&self, user: &User) -> String {
        let now = Utc::now();
        let iat = now.timestamp() as u64;
        let exp = (now + self.expires_in).timestamp() as u64;
        let claims = Claims {
            sub: user.id.to_string(),
            iat,
            exp,
        };

        jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )
        .expect("Token to be generated correctly")
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Claims {
    pub sub: String,
    pub iat: u64,
    pub exp: u64,
}

/// A user's verified identity provided by their server access token.
pub struct Identity {
    pub user_id: Uuid,
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

        let Ok(payload) = jsonwebtoken::decode::<Claims>(
            &token,
            &DecodingKey::from_secret(config.secret.as_ref()),
            &Validation::default(),
        ) else {
            return ready(Err(error::ErrorUnauthorized("Invalid Bearer token")));
        };

        let user_id = Uuid::parse_str(&payload.claims.sub).expect("sub claim to be a UUID");
        req.extensions_mut().insert::<Uuid>(user_id.to_owned());

        ready(Ok(Identity { user_id }))
    }
}
