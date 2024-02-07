use actix_web::{error, post, web, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use diesel::RunQueryDsl;
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{
    user::{InsertUser, User},
    DbError, DbPool,
};

pub fn config_service(cfg: &mut web::ServiceConfig) {
    cfg.service(sign_in);
}

const USER_INFO_REQUEST_URI: &'static str = "https://www.googleapis.com/userinfo/v2/me";

#[derive(Deserialize)]
#[allow(dead_code)]
struct UserInfoResponse {
    id: String,
    email: Option<String>,
    verified_email: bool,
    name: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
    picture: Option<String>,
    locale: Option<String>,
}

impl From<UserInfoResponse> for User {
    fn from(value: UserInfoResponse) -> Self {
        InsertUser {
            email: value.email,
            email_verified: value.verified_email,
            oauth2_id: Some(value.id),
            oauth2_name: value.name,
            oauth2_picture_url: value.picture,
            locale: value.locale,
        }
        .into()
    }
}

#[post("/sign_in/")]
async fn sign_in(
    pool: web::Data<DbPool>,
    authorization: BearerAuth,
) -> actix_web::Result<impl Responder> {
    let token = authorization.token();
    let client = reqwest::Client::new();
    let resp = client
        .get(USER_INFO_REQUEST_URI)
        .bearer_auth(token)
        .send()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let user_info = match resp.status() {
        StatusCode::OK => resp
            .json::<UserInfoResponse>()
            .await
            .map_err(error::ErrorInternalServerError),
        StatusCode::UNAUTHORIZED => Err(error::ErrorUnauthorized(resp.status())),
        _ => Err(error::ErrorInternalServerError(
            "Failed to get user information",
        )),
    }?;

    let added_user = web::block(move || {
        let mut conn = pool.get()?;

        use crate::schema::user::dsl::*;
        let new_user: User = user_info.into();

        Ok(diesel::insert_into(user)
            .values(&new_user)
            .on_conflict(email)
            .do_update()
            .set(&new_user)
            .get_result::<User>(&mut conn)?)
    })
    .await?
    .map_err(error::ErrorInternalServerError::<DbError>)?;

    Ok(HttpResponse::Ok().json(added_user))
}
