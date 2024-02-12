pub mod google_provider;
mod sign_in;

use crate::auth::oauth2::google_provider::{GoogleUserInfoService, RealGoogleUserInfoService};
use actix_web::web;
use std::sync::Arc;

pub fn config_service(cfg: &mut web::ServiceConfig) {
    let google_user_info_service =
        web::Data::from(Arc::new(RealGoogleUserInfoService) as Arc<dyn GoogleUserInfoService>);
    cfg.app_data(google_user_info_service)
        .service(sign_in::sign_in);
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
