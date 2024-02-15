mod sign_in;
mod verification;

use self::verification::{GameCenterIdValidationService, RealGameCenterIdValidationService};
use actix_web::web;
use std::sync::Arc;

pub fn config_service(cfg: &mut web::ServiceConfig) {
    let id_validation_service = web::Data::from(
        Arc::new(RealGameCenterIdValidationService) as Arc<dyn GameCenterIdValidationService>
    );
    cfg.app_data(id_validation_service)
        .service(sign_in::sign_in);
}
