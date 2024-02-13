use actix_web::{error, post, web, HttpResponse};
use serde::Deserialize;

use crate::auth::identity::IdentityConfig;
use crate::auth::refresh::session::{RefreshResult, RefreshSession};
use crate::db::DbPool;

pub mod session;

#[derive(Deserialize)]
struct RefreshRequestBody {
    refresh_token: String,
}

#[post("/refresh/")]
async fn refresh(
    pool: web::Data<DbPool>,
    identity_config: web::Data<IdentityConfig>,
    body: web::Json<RefreshRequestBody>,
) -> actix_web::Result<HttpResponse> {
    let mut conn = pool.get().await.map_err(error::ErrorInternalServerError)?;
    let result = RefreshSession::refresh(&mut conn, &identity_config, &body.refresh_token)
        .await
        .map_err(error::ErrorInternalServerError)?;

    match result {
        RefreshResult::Success(success) => Ok(HttpResponse::Ok().json(success)),
        _ => Ok(HttpResponse::BadRequest().finish()),
    }
}
