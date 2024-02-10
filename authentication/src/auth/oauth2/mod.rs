use self::google_provider::{GoogleUserInfoService, RealGoogleUserInfoService};
use crate::auth::identity::IdentityConfig;
use crate::auth::provider::{AuthProvider, AuthProviderChangeset, AuthProviderType};
use crate::auth::token::BearerToken;
use crate::user::{User, UserWithAuthProviders};
use crate::{schema, DbError, DbPool};
use actix_web::{cookie, error, post, web, HttpResponse, Responder};
use diesel::prelude::*;
use serde::Serialize;
use std::sync::Arc;

mod google_provider;

pub fn config_service(cfg: &mut web::ServiceConfig) {
    let google_user_info_service =
        web::Data::from(Arc::new(RealGoogleUserInfoService) as Arc<dyn GoogleUserInfoService>);
    cfg.app_data(google_user_info_service).service(sign_in);
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(serde::Deserialize, PartialEq))]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
enum SignInResult {
    Success(SignInSuccess),
    PendingLinkOrCreate,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(serde::Deserialize, PartialEq))]
struct SignInSuccess {
    server_token: String,
    user: UserWithAuthProviders,
}

#[post("/sign-in/")]
async fn sign_in(
    pool: web::Data<DbPool>,
    token: BearerToken,
    google_user_info_service: web::Data<dyn GoogleUserInfoService>,
    identity_config: web::Data<IdentityConfig>,
) -> actix_web::Result<impl Responder> {
    let user_info = google_user_info_service.get_info(&token).await?;

    let user_provider = web::block(move || {
        let mut conn = pool.get()?;

        let provider_exists = diesel::select(diesel::dsl::exists(
            schema::auth_provider::table
                .filter(schema::auth_provider::provider_id.eq(&user_info.id))
                .filter(schema::auth_provider::provider_type.eq(AuthProviderType::OAuth2)),
        ))
        .get_result::<bool>(&mut conn)?;

        if !provider_exists {
            return Ok(None);
        }

        let provider_changeset: AuthProviderChangeset = (&user_info).into();
        let provider: AuthProvider = diesel::update(schema::auth_provider::table)
            .filter(schema::auth_provider::provider_id.eq(&user_info.id))
            .filter(schema::auth_provider::provider_type.eq(AuthProviderType::OAuth2))
            .set(&provider_changeset)
            .get_result(&mut conn)?;

        let user: User = schema::user::table
            .filter(schema::user::id.eq(&provider.user_id))
            .first(&mut conn)?;

        Ok(Some((user, provider)))
    })
    .await?
    .map_err(error::ErrorInternalServerError::<DbError>)?;

    let Some((user, provider)) = user_provider else {
        return Ok(HttpResponse::Ok().json(SignInResult::PendingLinkOrCreate));
    };

    let token = identity_config.generate_identity(&user);

    let sign_in_cookie = cookie::Cookie::build("server_token", token.to_owned())
        .path("/")
        .max_age(cookie::time::Duration::seconds(
            identity_config.expires_in.num_seconds(),
        ))
        .http_only(true)
        .finish();

    Ok(HttpResponse::Ok()
        .cookie(sign_in_cookie)
        .json(SignInResult::Success(SignInSuccess {
            server_token: token,
            user: UserWithAuthProviders {
                user,
                providers: vec![provider],
            },
        })))
}

#[cfg(test)]
mod tests {
    use self::google_provider::GoogleUserInfo;
    use crate::config;
    use actix_web::{http::header::AUTHORIZATION, test, web, App};
    use reqwest::StatusCode;

    use super::*;

    mod sign_in {

        use super::*;

        #[actix_web::test]
        async fn test_no_google_token_should_be_unauthorized() {
            let pool = web::Data::new(config::initialize_db_pool(&config::get_db_url()));
            let identity_config = web::Data::new(config::get_identity_config());
            let google_user_info_service = web::Data::from(
                Arc::new(RealGoogleUserInfoService) as Arc<dyn GoogleUserInfoService>
            );

            let app = test::init_service(
                App::new()
                    .app_data(pool)
                    .app_data(identity_config)
                    .app_data(google_user_info_service)
                    .service(sign_in),
            )
            .await;

            let req = test::TestRequest::post().uri("/sign-in/").to_request();

            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        }

        #[actix_web::test]
        async fn test_non_existing_sign_in_should_be_pending_link_or_create() {
            struct NewUserGoogleUserInfoService;

            #[async_trait::async_trait]
            impl GoogleUserInfoService for NewUserGoogleUserInfoService {
                async fn get_info(&self, _token: &str) -> Result<GoogleUserInfo, error::Error> {
                    Ok(GoogleUserInfo {
                        id: "examplenonexistent".to_string(),
                        email: Some("example@nonexistent.com".to_string()),
                        verified_email: false,
                        family_name: None,
                        given_name: None,
                        name: None,
                        locale: None,
                        picture: None,
                    })
                }
            }

            let pool = web::Data::new(config::initialize_db_pool(&config::get_db_url()));
            let identity_config = web::Data::new(config::get_identity_config());
            let google_user_info_service = web::Data::from(
                Arc::new(NewUserGoogleUserInfoService) as Arc<dyn GoogleUserInfoService>
            );

            let app = test::init_service(
                App::new()
                    .app_data(pool)
                    .app_data(identity_config)
                    .app_data(google_user_info_service)
                    .service(sign_in),
            )
            .await;

            let req = test::TestRequest::post()
                .insert_header((AUTHORIZATION, "Bearer mock"))
                .uri("/sign-in/")
                .to_request();

            let resp = test::call_service(&app, req).await;
            assert!(resp.status().is_success());

            let body: SignInResult = test::read_body_json(resp).await;
            assert_eq!(body, SignInResult::PendingLinkOrCreate);
        }
    }
}

// #[post("/link-account/")]
// async fn link_account(
//     pool: web::Data<DbPool>,
//     authorization: BearerAuth,
//     jwt_config: web::Data<JwtConfig>,
// ) -> actix_web::Result<impl Responder> {
//     Ok(todo!())
// }

// #[post("/new-account/")]
// async fn new_account(
//     pool: web::Data<DbPool>,
//     authorization: BearerAuth,
//     jwt_config: web::Data<JwtConfig>,
// ) -> actix_web::Result<impl Responder> {
//     Ok(todo!())
// }

// #[derive(Serialize)]
// #[serde(tag = "type", content = "payload", rename_all = "snake_case")]
// enum CheckEmailResult {
//     NoExisting,
//     Existing(Vec<AuthProvider>),
// }

// #[get("/existing-email-providers/")]
// async fn existing_email_providers(
//     pool: web::Data<DbPool>,
//     authorization: BearerAuth,
// ) -> actix_web::Result<impl Responder> {
//     let provider_info = get_provider_info(&authorization).await?;

//     let Some(email_to_find) = provider_info.email else {
//         return Ok(HttpResponse::Ok().json(CheckEmailResult::NoExisting));
//     };

//     let matching_providers = web::block(move || {
//         let mut conn = pool.get()?;

//         let providers = schema::auth_provider::table
//             .filter(schema::auth_provider::email.eq(&email_to_find))
//             .select(AuthProvider::as_select())
//             .load(&mut conn)?;

//         Ok(providers)
//     })
//     .await?
//     .map_err(error::ErrorInternalServerError::<DbError>)?;

//     if matching_providers.is_empty() {
//         Ok(HttpResponse::Ok().json(CheckEmailResult::NoExisting))
//     } else {
//         Ok(HttpResponse::Ok().json(CheckEmailResult::Existing(matching_providers)))
//     }
// }
