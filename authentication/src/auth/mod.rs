pub mod identity;
pub mod oauth2;
pub mod provider;
pub mod refresh;
pub mod token;

use crate::user::UserWithAuthProviders;
use actix_web::{cookie, post, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Serialize;

pub fn config_service(cfg: &mut web::ServiceConfig) {
    cfg.service(sign_out)
        .service(refresh::refresh)
        .service(web::scope("/oauth2").configure(oauth2::config_service));
}

#[post("/sign-out/")]
async fn sign_out(_: identity::Identity) -> impl Responder {
    let clear_access = cookie::Cookie::build("access_token", "")
        .path("/")
        .max_age(cookie::time::Duration::seconds(-1))
        .http_only(true)
        .finish();

    let clear_refresh = cookie::Cookie::build("refresh_token", "")
        .path("/")
        .max_age(cookie::time::Duration::seconds(-1))
        .http_only(true)
        .finish();

    HttpResponse::Ok()
        .cookie(clear_access)
        .cookie(clear_refresh)
        .finish()
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(serde::Deserialize, PartialEq))]
pub struct SignInSuccess {
    access_token: Token,
    refresh_token: Token,
    user: UserWithAuthProviders,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(serde::Deserialize, PartialEq))]
pub struct Token {
    pub value: String,
    pub expires_at: DateTime<Utc>,
}
