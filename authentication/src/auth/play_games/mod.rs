mod exchange_auth_code;
mod play_games_api;
mod sign_in;

use self::{
    exchange_auth_code::{PlayGamesExchangeAuthCodeService, RealPlayGamesExchangeAuthCodeService},
    play_games_api::players::{PlayersService, RealPlayersService},
};
use actix_web::web;
use std::sync::Arc;

pub fn config_service(cfg: &mut web::ServiceConfig) {
    let exchange_auth_code_service =
        web::Data::from(Arc::new(RealPlayGamesExchangeAuthCodeService)
            as Arc<dyn PlayGamesExchangeAuthCodeService>);
    let players_service = web::Data::from(Arc::new(RealPlayersService) as Arc<dyn PlayersService>);
    cfg.app_data(exchange_auth_code_service)
        .app_data(players_service)
        .service(sign_in::sign_in);
}
