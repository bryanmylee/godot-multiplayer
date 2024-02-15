use crate::{diesel_insertable, schema, user::User};
use diesel::{
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    prelude::*,
    serialize::ToSql,
    sql_types,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

diesel_insertable! {
    #[derive(Queryable, Selectable, Insertable, AsChangeset, Associations)]
    #[diesel(belongs_to(User))]
    #[diesel(table_name = schema::auth_provider)]
    #[diesel(check_for_backend(Pg))]
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct AuthProvider {
        pub user_id: Uuid,
        pub order: i16,
        pub provider_type: AuthProviderType,
        pub provider_id: String,
        pub email: Option<String>,
        pub email_verified: bool,
        pub display_name: Option<String>,
        pub user_name: Option<String>,
        pub picture_url: Option<String>,
        pub locale: Option<String>,
    }
}

impl Default for AuthProviderInsert {
    fn default() -> Self {
        AuthProviderInsert {
            user_id: Uuid::new_v4(),
            order: 0,
            provider_id: "".to_string(),
            provider_type: AuthProviderType::OAuth2,
            email: None,
            email_verified: false,
            user_name: None,
            display_name: None,
            picture_url: None,
            locale: None,
        }
    }
}

pub trait IntoAuthProviderInsert {
    fn into_provider_insert(&self, user: &User) -> AuthProviderInsert;
}

#[derive(AsChangeset)]
#[diesel(table_name = schema::auth_provider)]
pub struct AuthProviderChangeset {
    pub order: i16,
    pub email: Option<String>,
    pub email_verified: bool,
    pub display_name: Option<String>,
    pub user_name: Option<String>,
    pub picture_url: Option<String>,
    pub locale: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = sql_types::Text)]
pub enum AuthProviderType {
    #[serde(rename = "oauth2")]
    OAuth2,
    #[serde(rename = "steam")]
    Steam,
    #[serde(rename = "game_center")]
    AppleGameCenter,
    #[serde(rename = "play_games")]
    GooglePlayGames,
}

impl FromSql<sql_types::Text, Pg> for AuthProviderType {
    fn from_sql(
        bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        match String::from_sql(bytes)?.as_ref() {
            "oauth2" => Ok(AuthProviderType::OAuth2),
            "steam" => Ok(AuthProviderType::Steam),
            "game_center" => Ok(AuthProviderType::AppleGameCenter),
            "play_games" => Ok(AuthProviderType::GooglePlayGames),
            _ => Err("Unknown `ProviderType` received".into()),
        }
    }
}

impl ToSql<sql_types::Text, Pg> for AuthProviderType {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        <str as ToSql<sql_types::Text, Pg>>::to_sql(
            match self {
                AuthProviderType::OAuth2 => "oauth2",
                AuthProviderType::Steam => "steam",
                AuthProviderType::AppleGameCenter => "game_center",
                AuthProviderType::GooglePlayGames => "play_games",
            },
            out,
        )
    }
}
