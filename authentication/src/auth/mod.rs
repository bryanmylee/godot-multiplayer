use crate::user::User;
use actix_web::{
    cookie, error, http, post, web, FromRequest, HttpMessage, HttpResponse, Responder,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};
use uuid::Uuid;

mod oauth2;

pub fn config_service(cfg: &mut web::ServiceConfig) {
    cfg.service(sign_out)
        .service(web::scope("/oauth2").configure(oauth2::config_service));
}

#[post("/sign_out/")]
async fn sign_out(_: JwtMiddleware) -> impl Responder {
    let logout_cookie = cookie::Cookie::build("server_token", "")
        .path("/")
        .max_age(cookie::time::Duration::seconds(-1))
        .http_only(true)
        .finish();

    HttpResponse::Ok().cookie(logout_cookie).finish()
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

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let Some(token) = req
            .headers()
            .get(http::header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .map(|h| h.split_at(7).1.to_string())
        else {
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
        req.extensions_mut().insert::<Uuid>(user_id.to_owned());

        ready(Ok(JwtMiddleware { user_id }))
    }
}

fn generate_jwt_token(user: &User, config: &JwtConfig) -> String {
    let now = Utc::now();
    let iat = now.timestamp() as u64;
    let exp = (now + config.expires_in).timestamp() as u64;
    let claims = Claims {
        sub: user.id.to_string(),
        iat,
        exp,
    };

    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_ref()),
    )
    .expect("Token to be generated correctly")
}
