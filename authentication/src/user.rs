use crate::{auth::provider::AuthProvider, diesel_insertable, schema, DbError, DbPool};
use actix_web::{error, get, web, HttpResponse, Responder};
use diesel::prelude::*;
use is_empty::IsEmpty;
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
    // cfg.service(get_user_by_id).service(find_user);
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

#[derive(Deserialize, IsEmpty)]
struct FindUserQueryParams {
    email: Option<String>,
}

// #[get("/")]
// async fn find_user(
//     pool: web::Data<DbPool>,
//     params: web::Query<FindUserQueryParams>,
// ) -> actix_web::Result<impl Responder> {
//     if params.is_empty() {
//         return Ok(HttpResponse::BadRequest().body("At least one query parameter is required"));
//     }

//     // Use `web::block` to offload blocking Diesel queries without blocking server thread.
//     let user = web::block(move || {
//         // Obtaining a connection from the pool is also potentially blocking.
//         let mut conn = pool.get()?;

//         use crate::schema::user::dsl::*;
//         let mut query = user.into_boxed();
//         if let Some(_email) = &params.email {
//             query = query.filter(email.eq(_email));
//         }
//         Ok(query.first::<User>(&mut conn).optional()?)
//     })
//     .await?
//     .map_err(error::ErrorInternalServerError::<DbError>)?;

//     Ok(match user {
//         Some(user) => HttpResponse::Ok().json(user),
//         None => HttpResponse::NotFound().body(format!("No user found")),
//     })
// }
