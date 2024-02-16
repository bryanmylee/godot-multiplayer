/*
 * https://partner.steamgames.com/doc/webapi/ISteamUser.
 */

use super::{STEAM_CONFIG, URI};
use actix_web::error;

#[async_trait::async_trait]
pub trait SteamUserService: Sync {
    /// https://partner.steamgames.com/doc/webapi/ISteamUser#GetPlayerSummaries
    ///
    /// # Arguments
    ///
    /// * `steam_ids` - A list of steam IDs (max of 100).
    async fn get_player_summaries(
        &self,
        steam_ids: &[&str],
    ) -> Result<Vec<get_player_summaries::Player>, error::Error> {
        let client = reqwest::Client::new();

        let resp = client
            .get(format!("{URI}/ISteamUser/GetPlayerSummaries/v2/"))
            .query(&[
                ("key", STEAM_CONFIG.web_api_key.to_owned()),
                ("steamids", steam_ids.join(",")),
            ])
            .send()
            .await
            .map_err(error::ErrorInternalServerError)?;

        let body: get_player_summaries::Body =
            resp.json().await.map_err(error::ErrorInternalServerError)?;

        match body.response {
            get_player_summaries::Response::Players(players) => Ok(players),
            get_player_summaries::Response::Error(err) => {
                Err(error::ErrorInternalServerError(err.error_description))
            }
        }
    }
}

pub struct RealSteamUserService;

impl SteamUserService for RealSteamUserService {}

mod get_player_summaries {
    use serde::{Deserialize, Serialize};

    use crate::{
        auth::provider::{
            AuthProviderChangeset, AuthProviderInsert, AuthProviderType, IntoAuthProviderInsert,
        },
        user::{User, UserInsert},
    };

    #[derive(Debug, Clone, Deserialize)]
    pub struct Body {
        pub response: Response,
    }

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Response {
        Players(Vec<Player>),
        Error(Error),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Player {
        #[serde(rename = "steamid")]
        pub steam_id: String,
        #[serde(rename = "communityvisibilitystate")]
        pub community_visibility_state: Option<u8>,
        #[serde(rename = "profilestate")]
        pub profile_state: Option<u8>,
        #[serde(rename = "personaname")]
        pub persona_name: String,
        #[serde(rename = "lastlogoff")]
        pub last_logoff_ts: Option<u64>,
        #[serde(rename = "profileurl")]
        pub profile_url: String,
        pub avatar: String,
        #[serde(rename = "avatarmedium")]
        pub avatar_medium: String,
        #[serde(rename = "avatarfull")]
        pub avatar_full: String,
    }

    impl From<&Player> for AuthProviderChangeset {
        fn from(value: &Player) -> Self {
            AuthProviderChangeset {
                order: 0,
                email: None,
                email_verified: false,
                display_name: Some(value.persona_name.clone()),
                user_name: Some(value.persona_name.clone()),
                picture_url: Some(value.avatar.clone()),
                locale: None,
            }
        }
    }

    impl From<&Player> for UserInsert {
        fn from(value: &Player) -> Self {
            UserInsert {
                name: Some(value.persona_name.clone()),
            }
        }
    }

    impl IntoAuthProviderInsert for Player {
        fn into_provider_insert(&self, user: &User) -> AuthProviderInsert {
            AuthProviderInsert {
                user_id: user.id,
                order: 0,
                provider_type: AuthProviderType::Steam,
                provider_id: self.steam_id.clone(),
                email: None,
                email_verified: false,
                display_name: Some(self.persona_name.clone()),
                user_name: Some(self.persona_name.clone()),
                picture_url: Some(self.avatar.clone()),
                locale: None,
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Error {
        #[serde(rename = "errorcode")]
        pub error_code: u32,
        #[serde(rename = "errordesc")]
        pub error_description: String,
    }
}
