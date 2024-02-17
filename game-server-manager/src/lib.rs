pub mod config;
pub mod game;

use actix_web::{error, FromRequest};
use config::SERVICE_KEY;
use std::{
    future::{ready, Ready},
    ops::Deref,
};

pub struct ServiceKey(String);

impl ServiceKey {
    fn validate(&self) -> Result<(), error::Error> {
        if self.0 == SERVICE_KEY.to_owned() {
            Ok(())
        } else {
            Err(error::ErrorUnauthorized("Invalid Service-Key header"))
        }
    }
}

impl FromRequest for ServiceKey {
    type Future = Ready<Result<Self, Self::Error>>;
    type Error = error::Error;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let Some(key) = req
            .headers()
            .get("Service-Key")
            .and_then(|h| h.to_str().ok())
        else {
            return ready(Err(error::ErrorUnauthorized("Missing Service-Key header")));
        };

        ready(Ok(ServiceKey(key.into())))
    }
}

impl Deref for ServiceKey {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<String> for ServiceKey {
    fn as_ref(&self) -> &String {
        &self.0
    }
}
