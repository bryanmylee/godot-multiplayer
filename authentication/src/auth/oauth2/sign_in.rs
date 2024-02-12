use crate::auth::identity::IdentityConfig;
use crate::auth::oauth2::google_provider::{GoogleUserInfo, GoogleUserInfoService};
use crate::auth::provider::{AuthProvider, AuthProviderChangeset, AuthProviderType};
use crate::auth::token::BearerToken;
use crate::auth::SignInSuccess;
use crate::db::{DbConnection, DbPool};
use crate::schema;
use crate::user::{User, UserInsert, UserWithAuthProviders};
use actix_web::{cookie, error, post, web, HttpResponse};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(serde::Deserialize, PartialEq))]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum SignInResult {
    Success(SignInSuccess),
    PendingLinkOrCreate(Vec<UserWithAuthProviders>),
}

#[post("/sign-in/")]
async fn sign_in(
    pool: web::Data<DbPool>,
    token: BearerToken,
    google_user_info_service: web::Data<dyn GoogleUserInfoService>,
    identity_config: web::Data<IdentityConfig>,
) -> actix_web::Result<HttpResponse> {
    let user_info = google_user_info_service.get_info(&token).await?;

    let mut conn = pool.get().await.map_err(error::ErrorInternalServerError)?;

    let provider_changeset: AuthProviderChangeset = (&user_info).into();
    let matching_provider: Option<AuthProvider> = diesel::update(schema::auth_provider::table)
        .filter(schema::auth_provider::provider_id.eq(&user_info.id))
        .filter(schema::auth_provider::provider_type.eq(AuthProviderType::OAuth2))
        .set(&provider_changeset)
        .get_result(&mut conn)
        .await
        .optional()
        .map_err(error::ErrorInternalServerError)?;

    if let Some(matching_provider) = matching_provider {
        let user: User = schema::user::table
            .filter(schema::user::id.eq(&matching_provider.user_id))
            .first(&mut conn)
            .await
            .map_err(error::ErrorInternalServerError)?;

        let providers = user
            .get_providers(&mut conn)
            .await
            .map_err(error::ErrorInternalServerError)?;

        return Ok(generate_sign_in_success_response(
            UserWithAuthProviders { user, providers },
            &identity_config,
        ));
    }

    let Some(email) = &user_info.email else {
        let new_user = create_new_user(&mut conn, &user_info)
            .await
            .map_err(error::ErrorInternalServerError)?;
        return Ok(generate_sign_in_success_response(
            new_user,
            &identity_config,
        ));
    };

    let same_email_providers: Vec<AuthProvider> = schema::auth_provider::table
        .filter(schema::auth_provider::email.eq(&email))
        .get_results(&mut conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    if same_email_providers.is_empty() {
        let new_user = create_new_user(&mut conn, &user_info)
            .await
            .map_err(error::ErrorInternalServerError)?;
        return Ok(generate_sign_in_success_response(
            new_user,
            &identity_config,
        ));
    }

    let user_ids: Vec<Uuid> = same_email_providers.iter().map(|p| p.user_id).collect();

    let users: Vec<User> = schema::user::table
        .filter(schema::user::id.eq_any(user_ids))
        .get_results(&mut conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let users_with_providers: Vec<UserWithAuthProviders> = same_email_providers
        .grouped_by(&users)
        .into_iter()
        .zip(users)
        .map(|(providers, user)| UserWithAuthProviders { user, providers })
        .collect();

    Ok(HttpResponse::Ok().json(SignInResult::PendingLinkOrCreate(users_with_providers)))
}

fn generate_sign_in_success_response(
    user_with_providers: UserWithAuthProviders,
    identity_config: &IdentityConfig,
) -> HttpResponse {
    let token = identity_config.generate_identity(&user_with_providers.user);

    let sign_in_cookie = cookie::Cookie::build("server_token", token.to_owned())
        .path("/")
        .max_age(cookie::time::Duration::seconds(
            identity_config.expires_in.num_seconds(),
        ))
        .http_only(true)
        .finish();

    HttpResponse::Ok()
        .cookie(sign_in_cookie)
        .json(SignInResult::Success(SignInSuccess {
            server_token: token,
            user: user_with_providers,
        }))
}

async fn create_new_user(
    conn: &mut DbConnection,
    user_info: &GoogleUserInfo,
) -> Result<UserWithAuthProviders, Box<dyn std::error::Error>> {
    let user_insert: UserInsert = user_info.into();
    let user = diesel::insert_into(schema::user::table)
        .values(user_insert)
        .get_result::<User>(conn)
        .await?;
    let provider = diesel::insert_into(schema::auth_provider::table)
        .values(user_info.into_provider_insert(&user))
        .get_result::<AuthProvider>(conn)
        .await?;
    Ok(UserWithAuthProviders {
        user,
        providers: vec![provider],
    })
}

#[cfg(test)]
mod tests {
    use crate::auth::provider::AuthProviderInsert;
    use crate::{config, db};
    use actix_web::{http::header::AUTHORIZATION, test, web, App};
    use reqwest::StatusCode;
    use std::sync::Arc;
    use uuid::Uuid;

    use super::*;

    #[actix_web::test]
    async fn test_no_google_token_should_be_unauthorized() {
        struct NeverGoogleUserInfoService;

        #[async_trait::async_trait]
        impl GoogleUserInfoService for NeverGoogleUserInfoService {
            async fn get_info(&self, _token: &str) -> Result<GoogleUserInfo, error::Error> {
                panic!("Unreachable code")
            }
        }

        let pool = web::Data::new(db::initialize_db_pool(&config::get_db_url()).await);
        let identity_config = web::Data::new(config::get_identity_config());
        let google_user_info_service =
            web::Data::from(Arc::new(NeverGoogleUserInfoService) as Arc<dyn GoogleUserInfoService>);

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
    async fn test_non_existing_provider_but_existing_email_should_be_pending() {
        lazy_static::lazy_static! {
            static ref EXISTING_NAME: String = "Adam".to_string();

            static ref EXISTING_EMAIL: String = "example@existing.com".to_string();

            static ref USER_INFO: GoogleUserInfo = GoogleUserInfo {
                id: "0001".to_string(),
                email: Some(EXISTING_EMAIL.clone()),
                verified_email: true,
                name: Some("Bryan".to_string()),
                family_name: None,
                given_name: Some("Bryan".to_string()),
                locale: None,
                picture: None,
            };
        }

        struct MockGoogleUserInfoService;

        #[async_trait::async_trait]
        impl GoogleUserInfoService for MockGoogleUserInfoService {
            async fn get_info(&self, _token: &str) -> Result<GoogleUserInfo, error::Error> {
                Ok(USER_INFO.clone())
            }
        }

        let pool = db::initialize_db_pool(&config::get_db_url()).await;
        {
            let mut conn = pool
                .get()
                .await
                .expect("Failed to get a database connection");

            let existing_user = User {
                id: Uuid::new_v4(),
                name: Some(EXISTING_NAME.clone()),
            };
            diesel::insert_into(schema::user::table)
                .values(&existing_user)
                .execute(&mut conn)
                .await
                .expect("Failed to insert user");

            let provider_insert = AuthProviderInsert {
                user_id: existing_user.id,
                email: Some(EXISTING_EMAIL.clone()),
                ..AuthProviderInsert::default()
            };
            diesel::insert_into(schema::auth_provider::table)
                .values(&provider_insert)
                .execute(&mut conn)
                .await
                .expect("Failed to insert provider");
        }

        let pool = web::Data::new(pool);
        let identity_config = web::Data::new(config::get_identity_config());
        let google_user_info_service =
            web::Data::from(Arc::new(MockGoogleUserInfoService) as Arc<dyn GoogleUserInfoService>);

        let app = test::init_service(
            App::new()
                .app_data(pool)
                .app_data(identity_config)
                .app_data(google_user_info_service)
                .service(sign_in),
        )
        .await;

        let req = test::TestRequest::post()
            .insert_header((AUTHORIZATION, "Bearer 0000"))
            .uri("/sign-in/")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: SignInResult = test::read_body_json(resp).await;
        assert!(matches!(body, SignInResult::PendingLinkOrCreate(_)));
        let SignInResult::PendingLinkOrCreate(matched_users) = body else {
            panic!()
        };

        assert_eq!(matched_users.len(), 1);
        let matched_user = matched_users[0].clone();

        assert_eq!(matched_user.user.name, Some(EXISTING_NAME.clone()));

        assert_eq!(matched_user.providers.len(), 1);
    }

    #[actix_web::test]
    async fn test_non_existing_provider_with_no_existing_email_should_sign_in() {
        lazy_static::lazy_static! {
            static ref USER_INFO: GoogleUserInfo = GoogleUserInfo {
                id: "examplenonexistent".to_string(),
                email: Some("example@nonexistent.com".to_string()),
                verified_email: false,
                family_name: None,
                given_name: None,
                name: None,
                locale: None,
                picture: None,
            };
        }

        struct MockGoogleUserInfoService;

        #[async_trait::async_trait]
        impl GoogleUserInfoService for MockGoogleUserInfoService {
            async fn get_info(&self, _token: &str) -> Result<GoogleUserInfo, error::Error> {
                Ok(USER_INFO.clone())
            }
        }

        let pool = web::Data::new(db::initialize_db_pool(&config::get_db_url()).await);
        let identity_config = web::Data::new(config::get_identity_config());
        let google_user_info_service =
            web::Data::from(Arc::new(MockGoogleUserInfoService) as Arc<dyn GoogleUserInfoService>);

        let app = test::init_service(
            App::new()
                .app_data(pool)
                .app_data(identity_config)
                .app_data(google_user_info_service)
                .service(sign_in),
        )
        .await;

        let req = test::TestRequest::post()
            .insert_header((AUTHORIZATION, "Bearer 0000"))
            .uri("/sign-in/")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: SignInResult = test::read_body_json(resp).await;

        assert!(matches!(body, SignInResult::Success(_)));
        let SignInResult::Success(success) = body else {
            panic!()
        };

        assert_eq!(success.user.user.name, USER_INFO.name);

        assert_eq!(success.user.providers.len(), 1);
        assert_eq!(
            success.user.providers[0].clone().into_insert(),
            USER_INFO.into_provider_insert(&success.user.user)
        );
    }

    #[actix_web::test]
    async fn test_existing_user_should_sign_in() {
        lazy_static::lazy_static! {
            static ref USER_INFO: GoogleUserInfo = GoogleUserInfo {
                id: "existing".to_string(),
                email: Some("example@existing.com".to_string()),
                verified_email: true,
                name: Some("Adam".to_string()),
                family_name: None,
                given_name: Some("Adam".to_string()),
                locale: None,
                picture: None,
            };
        }

        let existing_user = User {
            id: Uuid::new_v4(),
            name: Some("Adam".to_string()),
        };

        struct MockGoogleUserInfoService;

        #[async_trait::async_trait]
        impl GoogleUserInfoService for MockGoogleUserInfoService {
            async fn get_info(&self, _token: &str) -> Result<GoogleUserInfo, error::Error> {
                Ok(USER_INFO.clone())
            }
        }

        let pool = db::initialize_db_pool(&config::get_db_url()).await;
        {
            let mut conn = pool
                .get()
                .await
                .expect("Failed to get a database connection");

            diesel::insert_into(schema::user::table)
                .values(&existing_user)
                .execute(&mut conn)
                .await
                .expect("Failed to insert user");

            diesel::insert_into(schema::auth_provider::table)
                .values(USER_INFO.into_provider_insert(&existing_user))
                .execute(&mut conn)
                .await
                .expect("Failed to insert provider");
        }

        let pool = web::Data::new(pool);
        let identity_config = web::Data::new(config::get_identity_config());
        let google_user_info_service =
            web::Data::from(Arc::new(MockGoogleUserInfoService) as Arc<dyn GoogleUserInfoService>);

        let app = test::init_service(
            App::new()
                .app_data(pool)
                .app_data(identity_config)
                .app_data(google_user_info_service)
                .service(sign_in),
        )
        .await;

        let req = test::TestRequest::post()
            .insert_header((AUTHORIZATION, "Bearer 0000"))
            .uri("/sign-in/")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: SignInResult = test::read_body_json(resp).await;

        assert!(std::matches!(body, SignInResult::Success(_)));
        let SignInResult::Success(success) = body else {
            panic!()
        };

        assert_eq!(success.user.user, existing_user);

        assert_eq!(success.user.providers.len(), 1);
        assert_eq!(
            success.user.providers[0].clone().into_insert(),
            USER_INFO.into_provider_insert(&existing_user)
        );
    }
}
