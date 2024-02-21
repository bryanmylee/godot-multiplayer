/*
 * An interface for the Steamworks Web API.
 *
 * Refer to https://partner.steamgames.com/doc/webapi_overview.
 */

pub mod user;
pub mod user_auth;

const URL: &'static str = "https://partner.steam-api.com";

const AUTH_SERVER_STEAM_IDENTITY: &'static str = "authentication";
