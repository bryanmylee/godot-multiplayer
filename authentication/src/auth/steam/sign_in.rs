use crate::auth::identity::IdentityConfig;
use crate::auth::provider::{AuthProvider, AuthProviderChangeset, AuthProviderType};
use crate::auth::steam::steam_api::{user::SteamUserService, user_auth::SteamUserAuthService};
use crate::auth::{create_new_user, generate_sign_in_success_response};
use crate::db::DbPool;
use crate::schema;
use crate::user::{User, UserWithAuthProviders};
use actix_web::{error, post, web, HttpResponse};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

#[post("/sign-in/")]
async fn sign_in(
    auth_ticket: String,
    pool: web::Data<DbPool>,
    identity_config: web::Data<IdentityConfig>,
    user_service: web::Data<dyn SteamUserService>,
    user_auth_service: web::Data<dyn SteamUserAuthService>,
) -> actix_web::Result<HttpResponse> {
    let user_params = user_auth_service
        .authenticate_user_ticket(&auth_ticket)
        .await?;

    let user_info = user_service
        .get_player_summaries(&[&user_params.steam_id])
        .await?;
    let user_info = user_info
        .get(0)
        .expect(&format!("No user found with id {}", user_params.steam_id));

    let mut conn = pool.get().await.map_err(error::ErrorInternalServerError)?;

    let provider_changeset: AuthProviderChangeset = user_info.into();
    let matching_provider: Option<AuthProvider> = diesel::update(schema::auth_provider::table)
        .filter(schema::auth_provider::provider_id.eq(&user_info.steam_id))
        .filter(schema::auth_provider::provider_type.eq(AuthProviderType::Steam))
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

        return generate_sign_in_success_response(
            &mut conn,
            UserWithAuthProviders { user, providers },
            &identity_config,
        )
        .await;
    }

    let new_user = create_new_user(&mut conn, user_info)
        .await
        .map_err(error::ErrorInternalServerError)?;

    generate_sign_in_success_response(&mut conn, new_user, &identity_config).await
}
