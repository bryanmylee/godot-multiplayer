use actix_web::web;

mod kill;
mod list;
mod spawn;

pub fn config_service(cfg: &mut web::ServiceConfig) {
    cfg.service(list::list);
}
