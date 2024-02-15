use crate::auth::game_center::id_validation::{GameCenterIdValidationService, IdentitySignature};
use crate::auth::identity::IdentityConfig;
use crate::auth::provider::{AuthProvider, AuthProviderChangeset, AuthProviderType};
use crate::auth::{create_new_user, generate_sign_in_success_response};
use crate::db::DbPool;
use crate::schema;
use crate::user::{User, UserWithAuthProviders};
use actix_web::{error, post, web, HttpResponse};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

#[post("/sign-in/")]
async fn sign_in(
    id_signature: web::Json<IdentitySignature>,
    pool: web::Data<DbPool>,
    identity_config: web::Data<IdentityConfig>,
    id_validation_service: web::Data<dyn GameCenterIdValidationService>,
) -> actix_web::Result<HttpResponse> {
    let id_signature = id_signature.0;
    let validated = id_validation_service.is_validated(&id_signature).await?;
    if !validated {
        return Err(error::ErrorUnauthorized("Failed to validate identity"));
    }

    let mut conn = pool.get().await.map_err(error::ErrorInternalServerError)?;

    let provider_changeset: AuthProviderChangeset = (&id_signature).into();
    let matching_provider: Option<AuthProvider> = diesel::update(schema::auth_provider::table)
        .filter(schema::auth_provider::provider_id.eq(&id_signature.player_id))
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

        return generate_sign_in_success_response(
            &mut conn,
            UserWithAuthProviders { user, providers },
            &identity_config,
        )
        .await;
    }

    let new_user = create_new_user(&mut conn, &id_signature)
        .await
        .map_err(error::ErrorInternalServerError)?;
    return generate_sign_in_success_response(&mut conn, new_user, &identity_config).await;
}
