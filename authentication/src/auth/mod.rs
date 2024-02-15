pub mod game_center;
pub mod identity;
pub mod oauth2;
pub mod play_games;
pub mod provider;
pub mod refresh;
pub mod token;

use crate::auth::identity::{Identity, IdentityConfig};
use crate::auth::provider::{AuthProvider, IntoAuthProviderInsert};
use crate::auth::refresh::session::RefreshSession;
use crate::schema;
use crate::user::{User, UserInsert};
use crate::{db::DbConnection, user::UserWithAuthProviders};
use actix_web::{cookie, error, post, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use diesel_async::RunQueryDsl;
use serde::Serialize;

pub fn config_service(cfg: &mut web::ServiceConfig) {
    cfg.service(sign_out)
        .service(refresh::refresh)
        .service(web::scope("/oauth2").configure(oauth2::config_service))
        .service(web::scope("/game-center").configure(game_center::config_service))
        .service(web::scope("/play-games").configure(play_games::config_service));
}

#[post("/sign-out/")]
async fn sign_out(_: identity::Identity) -> impl Responder {
    let clear_access = cookie::Cookie::build("access_token", "")
        .path("/")
        .max_age(cookie::time::Duration::seconds(-1))
        .http_only(true)
        .finish();

    let clear_refresh = cookie::Cookie::build("refresh_token", "")
        .path("/")
        .max_age(cookie::time::Duration::seconds(-1))
        .http_only(true)
        .finish();

    HttpResponse::Ok()
        .cookie(clear_access)
        .cookie(clear_refresh)
        .finish()
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(serde::Deserialize, PartialEq))]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum SignInResult {
    Success(SignInSuccess),
    PendingLinkOrCreate(Vec<UserWithAuthProviders>),
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(serde::Deserialize, PartialEq))]
pub struct SignInSuccess {
    access_token: Token,
    refresh_token: Token,
    user: UserWithAuthProviders,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(serde::Deserialize, PartialEq))]
pub struct Token {
    pub value: String,
    pub expires_at: DateTime<Utc>,
}

pub async fn generate_sign_in_success_response(
    conn: &mut DbConnection,
    user_with_providers: UserWithAuthProviders,
    identity_config: &IdentityConfig,
) -> actix_web::Result<HttpResponse> {
    let identity = Identity {
        user_id: user_with_providers.user.id,
    };
    let access_token = identity.generate_token(identity_config);
    let sign_in_cookie = cookie::Cookie::build("access_token", access_token.value.to_owned())
        .path("/")
        .max_age(cookie::time::Duration::seconds(
            identity_config.expires_in.num_seconds(),
        ))
        .http_only(true)
        .finish();

    let refresh_session =
        RefreshSession::create(conn, &identity_config, &user_with_providers.user.id)
            .await
            .map_err(error::ErrorInternalServerError)?;
    let refresh_token = refresh_session.generate_token(&identity_config);
    let refresh_cookie = cookie::Cookie::build("refresh_token", refresh_token.value.to_owned())
        .path("/")
        .max_age(cookie::time::Duration::seconds(
            identity_config.refresh_expires_in.num_seconds(),
        ))
        .http_only(true)
        .finish();

    Ok(HttpResponse::Ok()
        .cookie(sign_in_cookie)
        .cookie(refresh_cookie)
        .json(SignInResult::Success(SignInSuccess {
            access_token,
            refresh_token,
            user: user_with_providers,
        })))
}

pub async fn create_new_user<I>(
    conn: &mut DbConnection,
    user_info: &I,
) -> Result<UserWithAuthProviders, Box<dyn std::error::Error>>
where
    I: IntoAuthProviderInsert,
    for<'a> &'a I: Into<UserInsert>,
{
    let user_insert: UserInsert = user_info.into();
    let user: User = diesel::insert_into(schema::user::table)
        .values(user_insert)
        .get_result(conn)
        .await?;
    let provider: AuthProvider = diesel::insert_into(schema::auth_provider::table)
        .values(user_info.into_provider_insert(&user))
        .get_result(conn)
        .await?;
    Ok(UserWithAuthProviders {
        user,
        providers: vec![provider],
    })
}
