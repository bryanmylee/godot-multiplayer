/*
 * https://partner.steamgames.com/doc/webapi/ISteamUserAuth.
 */

use super::{AUTH_SERVER_STEAM_IDENTITY, STEAM_CONFIG, URI};
use actix_web::error;

#[async_trait::async_trait]
pub trait SteamUserAuthService: Sync {
    /// https://partner.steamgames.com/doc/webapi/ISteamUserAuth#AuthenticateUserTicket
    ///
    /// # Arguments
    ///
    /// * `ticket` - The ticket data from GetAuthTicketForWebApi encoded as a hexadecimal string.created.
    async fn authenticate_user_ticket(
        &self,
        ticket: &str,
    ) -> Result<authenticate_user_ticket::Params, error::Error> {
        let client = reqwest::Client::new();

        let resp = client
            .get(format!("{URI}/ISteamUserAuth/AuthenticateUserTicket/v1/"))
            .query(&[
                ("key", STEAM_CONFIG.web_api_key.to_owned()),
                ("appid", STEAM_CONFIG.app_id.to_owned()),
                ("ticket", ticket.to_owned()),
                ("identity", AUTH_SERVER_STEAM_IDENTITY.to_owned()),
            ])
            .send()
            .await
            .map_err(error::ErrorInternalServerError)?;

        let body: authenticate_user_ticket::Body =
            resp.json().await.map_err(error::ErrorInternalServerError)?;

        match body.response {
            authenticate_user_ticket::Response::Params(params) => Ok(params),
            authenticate_user_ticket::Response::Error(err) => {
                Err(error::ErrorUnauthorized(err.error_description))
            }
        }
    }
}

pub struct RealSteamUserAuthService;

impl SteamUserAuthService for RealSteamUserAuthService {}

mod authenticate_user_ticket {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Deserialize)]
    pub struct Body {
        pub response: Response,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Response {
        Params(Params),
        Error(Error),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Params {
        pub result: String,
        #[serde(rename = "steamid")]
        pub steam_id: String,
        #[serde(rename = "ownersteamid")]
        pub owner_steam_id: String,
        #[serde(rename = "vacbanned")]
        pub vac_banned: bool,
        #[serde(rename = "publisherbanned")]
        pub publisher_banned: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Error {
        #[serde(rename = "errorcode")]
        pub error_code: u32,
        #[serde(rename = "errordesc")]
        pub error_description: String,
    }
}
