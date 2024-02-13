pub mod identity;
pub mod oauth2;
pub mod provider;
pub mod refresh;
pub mod token;

use crate::user::UserWithAuthProviders;
use actix_web::{cookie, post, web, HttpResponse, Responder};
use serde::Serialize;

pub fn config_service(cfg: &mut web::ServiceConfig) {
    cfg.service(sign_out)
        .service(refresh::refresh)
        .service(web::scope("/oauth2").configure(oauth2::config_service));
}

#[post("/sign-out/")]
async fn sign_out(_: identity::Identity) -> impl Responder {
    let logout_cookie = cookie::Cookie::build("access_token", "")
        .path("/")
        .max_age(cookie::time::Duration::seconds(-1))
        .http_only(true)
        .finish();

    HttpResponse::Ok().cookie(logout_cookie).finish()
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(serde::Deserialize, PartialEq))]
pub struct SignInSuccess {
    access_token: String,
    refresh_token: String,
    user: UserWithAuthProviders,
}
