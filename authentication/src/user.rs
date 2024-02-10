use crate::db::{DbError, DbPool};
use crate::{auth::provider::AuthProvider, diesel_insertable, schema};
use actix_web::{error, get, web, HttpResponse, Responder};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

diesel_insertable! {
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Queryable, Selectable, Insertable)]
    #[diesel(table_name = crate::schema::user)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    pub struct User {
        pub name: Option<String>,
    }
}

pub fn config_service(cfg: &mut web::ServiceConfig) {
    cfg.service(get_user_by_id);
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(Deserialize, PartialEq))]
pub struct UserWithAuthProviders {
    #[serde(flatten)]
    pub user: User,
    pub providers: Vec<AuthProvider>,
}

#[get("/{user_id}/")]
async fn get_user_by_id(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
) -> actix_web::Result<impl Responder> {
    let id_to_find = path.into_inner();

    let user = web::block(move || {
        let mut conn = pool.get()?;

        let found_user = schema::user::table
            .filter(schema::user::id.eq(&id_to_find))
            .first::<User>(&mut conn)
            .optional()?;

        let Some(found_user) = found_user else {
            return Ok(None);
        };

        let providers = AuthProvider::belonging_to(&found_user)
            .select(AuthProvider::as_select())
            .load(&mut conn)?;

        Ok(Some(UserWithAuthProviders {
            user: found_user,
            providers,
        }))
    })
    .await?
    .map_err(error::ErrorInternalServerError::<DbError>)?;

    Ok(match user {
        Some(user) => HttpResponse::Ok().json(user),
        None => HttpResponse::NotFound().body(format!("No user found with id {id_to_find}")),
    })
}
