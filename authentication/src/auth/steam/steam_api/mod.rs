/*
 * An interface for the Steamworks Web API.
 *
 * Refer to https://partner.steamgames.com/doc/webapi_overview.
 */

use crate::config::{get_steam_config, SteamConfig};

pub mod user;
pub mod user_auth;

const URI: &'static str = "https://partner.steam-api.com";

lazy_static::lazy_static! {
  static ref STEAM_CONFIG: SteamConfig = get_steam_config();
}

const AUTH_SERVER_STEAM_IDENTITY: &'static str = "authentication";
