use crate::auth::provider::{AuthProvider, AuthProviderChangeset, AuthProviderType};
use crate::auth::{generate_jwt_token, JwtConfig};
use crate::user::{UserInsert, UserWithAuthProviders};
use crate::{schema, user::User, DbError, DbPool};
use actix_web::{cookie, error, get, post, web, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use diesel::prelude::*;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use super::provider::AuthProviderInsert;

pub fn config_service(cfg: &mut web::ServiceConfig) {
    cfg.service(existing_email_providers).service(sign_in);
}

#[derive(Serialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
enum CheckEmailResult {
    NoExisting,
    Existing(Vec<AuthProvider>),
}

#[get("/existing-email-providers/")]
async fn existing_email_providers(
    pool: web::Data<DbPool>,
    authorization: BearerAuth,
) -> actix_web::Result<impl Responder> {
    let provider_info = get_provider_info(&authorization).await?;

    let Some(email_to_find) = provider_info.email else {
        return Ok(HttpResponse::Ok().json(CheckEmailResult::NoExisting));
    };

    let matching_providers = web::block(move || {
        let mut conn = pool.get()?;

        let providers = schema::auth_provider::table
            .filter(schema::auth_provider::email.eq(&email_to_find))
            .select(AuthProvider::as_select())
            .load(&mut conn)?;

        Ok(providers)
    })
    .await?
    .map_err(error::ErrorInternalServerError::<DbError>)?;

    if matching_providers.is_empty() {
        Ok(HttpResponse::Ok().json(CheckEmailResult::NoExisting))
    } else {
        Ok(HttpResponse::Ok().json(CheckEmailResult::Existing(matching_providers)))
    }
}

#[derive(Serialize)]
struct SignInSuccess {
    server_token: String,
    user: UserWithAuthProviders,
}

#[post("/sign-in/")]
async fn sign_in(
    pool: web::Data<DbPool>,
    authorization: BearerAuth,
    jwt_config: web::Data<JwtConfig>,
) -> actix_web::Result<impl Responder> {
    let provider_info = get_provider_info(&authorization).await?;

    let (user, provider) = web::block(move || {
        let mut conn = pool.get()?;

        let provider_exists = diesel::select(diesel::dsl::exists(
            schema::auth_provider::table
                .filter(schema::auth_provider::provider_id.eq(&provider_info.id))
                .filter(schema::auth_provider::provider_type.eq(AuthProviderType::OAuth2)),
        ))
        .get_result::<bool>(&mut conn)?;

        if provider_exists {
            let provider_changeset: AuthProviderChangeset = (&provider_info).into();
            let provider = diesel::update(schema::auth_provider::table)
                .filter(schema::auth_provider::provider_id.eq(&provider_info.id))
                .set(&provider_changeset)
                .get_result::<AuthProvider>(&mut conn)?;

            let user = schema::user::table
                .filter(schema::user::id.eq(&provider.user_id))
                .first::<User>(&mut conn)?;

            Ok((user, provider))
        } else {
            let user_insert: UserInsert = (&provider_info).into();
            let new_user = diesel::insert_into(schema::user::table)
                .values(&user_insert)
                .get_result::<User>(&mut conn)?;

            let provider_insert = provider_info.into_insert(&new_user);
            let provider = diesel::insert_into(schema::auth_provider::table)
                .values(provider_insert)
                .get_result::<AuthProvider>(&mut conn)?;

            Ok((new_user, provider))
        }
    })
    .await?
    .map_err(error::ErrorInternalServerError::<DbError>)?;

    let token = generate_jwt_token(&user, &jwt_config);

    let sign_in_cookie = cookie::Cookie::build("server_token", token.to_owned())
        .path("/")
        .max_age(cookie::time::Duration::seconds(
            jwt_config.expires_in.num_seconds(),
        ))
        .http_only(true)
        .finish();

    Ok(HttpResponse::Ok()
        .cookie(sign_in_cookie)
        .json(SignInSuccess {
            server_token: token,
            user: UserWithAuthProviders {
                user,
                providers: vec![provider],
            },
        }))
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct OAuthProviderInfo {
    id: String,
    email: Option<String>,
    verified_email: bool,
    name: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
    picture: Option<String>,
    locale: Option<String>,
}

impl From<&OAuthProviderInfo> for AuthProviderChangeset {
    fn from(value: &OAuthProviderInfo) -> Self {
        AuthProviderChangeset {
            order: 0,
            email: value.email.clone(),
            email_verified: value.verified_email,
            display_name: value.name.clone(),
            user_name: value.email.clone(),
            picture_url: value.picture.clone(),
            locale: value.locale.clone(),
        }
    }
}

impl From<&OAuthProviderInfo> for UserInsert {
    fn from(value: &OAuthProviderInfo) -> Self {
        UserInsert {
            name: value.name.clone(),
        }
    }
}

impl OAuthProviderInfo {
    fn into_insert(self: &Self, user: &User) -> AuthProviderInsert {
        AuthProviderInsert {
            user_id: user.id,
            order: 0,
            provider_type: AuthProviderType::OAuth2,
            provider_id: self.id.clone(),
            email: self.email.clone(),
            email_verified: self.verified_email,
            display_name: self.name.clone(),
            user_name: self.email.clone(),
            picture_url: self.picture.clone(),
            locale: self.locale.clone(),
        }
    }
}

const USER_INFO_REQUEST_URI: &'static str = "https://www.googleapis.com/userinfo/v2/me";

async fn get_provider_info(authorization: &BearerAuth) -> Result<OAuthProviderInfo, error::Error> {
    let token = authorization.token();
    let client = reqwest::Client::new();
    let resp = client
        .get(USER_INFO_REQUEST_URI)
        .bearer_auth(token)
        .send()
        .await
        .map_err(error::ErrorInternalServerError)?;

    match resp.status() {
        StatusCode::OK => resp
            .json::<OAuthProviderInfo>()
            .await
            .map_err(error::ErrorInternalServerError),
        StatusCode::UNAUTHORIZED => Err(error::ErrorUnauthorized(resp.status())),
        _ => Err(error::ErrorInternalServerError(
            "Failed to get user information",
        )),
    }
}
