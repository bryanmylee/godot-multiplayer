use actix_web::{error, http, FromRequest};
use std::{
    future::{ready, Ready},
    ops::Deref,
};

pub struct BearerToken(String);

impl FromRequest for BearerToken {
    type Future = Ready<Result<Self, Self::Error>>;
    type Error = error::Error;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let Some(token) = req
            .headers()
            .get(http::header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .map(|h| h.split_at(7).1.to_string())
        else {
            return ready(Err(error::ErrorUnauthorized("Missing Bearer token")));
        };

        ready(Ok(BearerToken(token)))
    }
}

impl Deref for BearerToken {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<String> for BearerToken {
    fn as_ref(&self) -> &String {
        &self.0
    }
}
