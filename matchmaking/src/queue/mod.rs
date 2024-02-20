mod join;
mod leave;

use actix_web::web;

pub fn config_service(cfg: &mut web::ServiceConfig) {
    cfg.service(join::join).service(leave::leave);
}
