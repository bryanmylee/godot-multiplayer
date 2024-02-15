pub mod google_user_info_api;
mod sign_in;

use self::google_user_info_api::{GoogleUserInfoService, RealGoogleUserInfoService};
use actix_web::web;
use std::sync::Arc;

pub fn config_service(cfg: &mut web::ServiceConfig) {
    let google_user_info_service =
        web::Data::from(Arc::new(RealGoogleUserInfoService) as Arc<dyn GoogleUserInfoService>);
    cfg.app_data(google_user_info_service)
        .service(sign_in::sign_in);
}
