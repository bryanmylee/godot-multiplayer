use crate::{diesel_insertable, DbError, DbPool};
use actix_web::{error, get, web, HttpResponse, Responder};
use diesel::prelude::*;
use is_empty::IsEmpty;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

diesel_insertable! {
    #[derive(Serialize, Deserialize, Debug, Clone, Identifiable, Queryable, Selectable, Insertable, AsChangeset)]
    #[diesel(table_name = crate::schema::user)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    pub struct User {
        pub email: Option<String>,
        pub email_verified: bool,
        pub locale: Option<String>,
        pub oauth2_id: Option<String>,
        pub oauth2_name: Option<String>,
        pub oauth2_picture_url: Option<String>,
    }
}

pub fn config_service(cfg: &mut web::ServiceConfig) {
    cfg.service(get_user_by_id).service(find_user);
}

#[get("/{user_id}/")]
async fn get_user_by_id(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
) -> actix_web::Result<impl Responder> {
    let user_id = path.into_inner();

    let user = web::block(move || {
        let mut conn = pool.get()?;

        use crate::schema::user::dsl::*;
        let mut query = user.into_boxed();
        query = query.filter(id.eq(user_id));
        Ok(query.first::<User>(&mut conn).optional()?)
    })
    .await?
    .map_err(error::ErrorInternalServerError::<DbError>)?;

    Ok(match user {
        Some(user) => HttpResponse::Ok().json(user),
        None => HttpResponse::NotFound().body(format!("No user found with id {user_id}")),
    })
}

#[derive(Deserialize, IsEmpty)]
struct FindUserQueryParams {
    email: Option<String>,
}

#[get("/")]
async fn find_user(
    pool: web::Data<DbPool>,
    params: web::Query<FindUserQueryParams>,
) -> actix_web::Result<impl Responder> {
    if params.is_empty() {
        return Ok(HttpResponse::BadRequest().body("At least one query parameter is required"));
    }

    // Use `web::block` to offload blocking Diesel queries without blocking server thread.
    let user = web::block(move || {
        // Obtaining a connection from the pool is also potentially blocking.
        let mut conn = pool.get()?;

        use crate::schema::user::dsl::*;
        let mut query = user.into_boxed();
        if let Some(_email) = &params.email {
            query = query.filter(email.eq(_email));
        }
        Ok(query.first::<User>(&mut conn).optional()?)
    })
    .await?
    .map_err(error::ErrorInternalServerError::<DbError>)?;

    Ok(match user {
        Some(user) => HttpResponse::Ok().json(user),
        None => HttpResponse::NotFound().body(format!("No user found")),
    })
}
