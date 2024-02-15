use crate::auth::identity::IdentityConfig;
use crate::auth::play_games::{
    exchange_auth_code::PlayGamesExchangeAuthCodeService, play_games_api::players::PlayersService,
};
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
    auth_code: String,
    pool: web::Data<DbPool>,
    identity_config: web::Data<IdentityConfig>,
    exchange_auth_code_service: web::Data<dyn PlayGamesExchangeAuthCodeService>,
    players_service: web::Data<dyn PlayersService>,
) -> actix_web::Result<HttpResponse> {
    let access_token = exchange_auth_code_service
        .get_access_token(&auth_code)
        .await?;

    let player = players_service.me(&access_token).await?;

    let mut conn = pool.get().await.map_err(error::ErrorInternalServerError)?;

    let provider_changeset: AuthProviderChangeset = (&player).into();
    let matching_provider: Option<AuthProvider> = diesel::update(schema::auth_provider::table)
        .filter(schema::auth_provider::provider_id.eq(&player.player_id))
        .filter(schema::auth_provider::provider_type.eq(AuthProviderType::GooglePlayGames))
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

    let new_user = create_new_user(&mut conn, &player)
        .await
        .map_err(error::ErrorInternalServerError)?;

    generate_sign_in_success_response(&mut conn, new_user, &identity_config).await
}
