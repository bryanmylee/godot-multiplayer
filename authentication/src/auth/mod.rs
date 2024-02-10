pub mod identity;
mod oauth2;
pub mod provider;
pub mod token;

use actix_web::{cookie, post, web, HttpResponse, Responder};

pub fn config_service(cfg: &mut web::ServiceConfig) {
    cfg.service(sign_out)
        .service(web::scope("/oauth2").configure(oauth2::config_service));
}

#[post("/sign-out/")]
async fn sign_out(_: identity::Identity) -> impl Responder {
    let logout_cookie = cookie::Cookie::build("server_token", "")
        .path("/")
        .max_age(cookie::time::Duration::seconds(-1))
        .http_only(true)
        .finish();

    HttpResponse::Ok().cookie(logout_cookie).finish()
}
