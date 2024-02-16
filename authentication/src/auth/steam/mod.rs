mod sign_in;
mod steam_api;

use self::steam_api::{
    user::{RealSteamUserService, SteamUserService},
    user_auth::{RealSteamUserAuthService, SteamUserAuthService},
};
use actix_web::web;
use std::sync::Arc;

pub fn config_service(cfg: &mut web::ServiceConfig) {
    let steam_user_service =
        web::Data::from(Arc::new(RealSteamUserService) as Arc<dyn SteamUserService>);
    let steam_user_auth_service =
        web::Data::from(Arc::new(RealSteamUserAuthService) as Arc<dyn SteamUserAuthService>);
    cfg.app_data(steam_user_service)
        .app_data(steam_user_auth_service)
        .service(sign_in::sign_in);
}
