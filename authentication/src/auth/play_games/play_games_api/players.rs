use super::URL;
use crate::{
    auth::provider::{
        AuthProviderChangeset, AuthProviderInsert, AuthProviderType, IntoAuthProviderInsert,
    },
    user::{User, UserInsert},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[async_trait::async_trait]
pub trait PlayersService: Sync {
    async fn me(&self, access_token: &str) -> Result<Player, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("{URL}/games/v1/players/me"))
            .timeout(Duration::from_secs(5))
            .bearer_auth(access_token)
            .send()
            .await?;
        let player: Player = resp.json().await?;
        Ok(player)
    }
}

pub struct RealPlayersService;

impl PlayersService for RealPlayersService {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub kind: String,
    pub player_id: String,
    pub display_name: Option<String>,
    pub avatar_image_url: Option<String>,
    pub banner_url_portrait: Option<String>,
    pub banner_url_landscape: Option<String>,
    pub original_player_id: Option<String>,
    pub profile_settings: Option<ProfileSettings>,
    pub name: Option<Name>,
    pub experience_info: Option<PlayerExperienceInfo>,
    pub title: Option<String>,
    pub friend_status: Option<FriendStatus>,
    pub game_player_id: Option<String>,
}

impl From<&Player> for AuthProviderChangeset {
    fn from(value: &Player) -> Self {
        AuthProviderChangeset {
            order: 0,
            email: None,
            email_verified: false,
            display_name: value.display_name.clone(),
            user_name: value.display_name.clone(),
            picture_url: value.avatar_image_url.clone(),
            locale: None,
        }
    }
}

impl From<&Player> for UserInsert {
    fn from(value: &Player) -> Self {
        UserInsert {
            name: value.display_name.clone(),
        }
    }
}

impl IntoAuthProviderInsert for Player {
    fn into_provider_insert(&self, user: &User) -> AuthProviderInsert {
        AuthProviderInsert {
            user_id: user.id,
            order: 0,
            provider_type: AuthProviderType::GooglePlayGames,
            provider_id: self.player_id.clone(),
            email: None,
            email_verified: false,
            display_name: self.display_name.clone(),
            user_name: self.display_name.clone(),
            picture_url: self.avatar_image_url.clone(),
            locale: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Name {
    pub family_name: Option<String>,
    pub given_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FriendsListVisibility {
    Visible,
    RequestRequired,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSettings {
    pub kind: String,
    pub profile_visible: bool,
    pub friends_list_visibility: Option<FriendsListVisibility>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerLevel {
    pub kind: String,
    pub level: i32,
    pub min_experience_points: String,
    pub max_experience_points: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerExperienceInfo {
    pub kind: String,
    pub current_experience_points: String,
    pub last_level_up_timestamp_millis: String,
    pub current_level: Option<PlayerLevel>,
    pub next_level: Option<PlayerLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FriendStatus {
    NoRelationship,
    Friend,
}
