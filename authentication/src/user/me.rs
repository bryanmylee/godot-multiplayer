use crate::auth::identity::Identity;
use crate::db::DbPool;
use crate::schema;
use crate::user::{User, UserWithAuthProviders};
use actix_web::{error, get, web, HttpResponse};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

#[get("/me/")]
async fn me(pool: web::Data<DbPool>, identity: Identity) -> actix_web::Result<HttpResponse> {
    let id_to_find = identity.user_id;

    let mut conn = pool.get().await.map_err(error::ErrorInternalServerError)?;

    let user = schema::user::table
        .filter(schema::user::id.eq(&id_to_find))
        .first::<User>(&mut conn)
        .await
        .optional()
        .map_err(error::ErrorInternalServerError)?;

    let Some(user) = user else {
        return Ok(HttpResponse::NotFound().body(format!("No user found with id {id_to_find}")));
    };

    let providers = user
        .get_providers(&mut conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(UserWithAuthProviders { user, providers }))
}

#[cfg(test)]
mod tests {
    use crate::auth::{oauth2::google_provider::GoogleUserInfo, provider::AuthProvider};
    use crate::{config, db};
    use actix_web::{http::header::AUTHORIZATION, test, web, App};
    use uuid::Uuid;

    use super::*;

    #[actix_web::test]
    async fn test_me_returns_the_correct_user() {
        lazy_static::lazy_static! {
            static ref USER_INFO: GoogleUserInfo = GoogleUserInfo {
                id: "0001".to_string(),
                email: Some("example@existing.com".to_string()),
                verified_email: true,
                name: Some("Adam".to_string()),
                family_name: None,
                given_name: Some("Adam".to_string()),
                locale: None,
                picture: None,
            };
        }

        let user = User {
            id: Uuid::new_v4(),
            name: Some("Adam".to_string()),
        };

        let main_provider = USER_INFO.into_provider_insert(&user).into_row();

        let pool = db::initialize_db_pool(&config::get_db_url()).await;
        {
            let mut conn = pool
                .get()
                .await
                .expect("Failed to get a database connection");

            diesel::insert_into(schema::user::table)
                .values(&user)
                .execute(&mut conn)
                .await
                .expect("Failed to insert user");

            diesel::insert_into(schema::auth_provider::table)
                .values(&main_provider)
                .execute(&mut conn)
                .await
                .expect("Failed to insert provider");
        }

        let pool = web::Data::new(pool);
        let identity_config = web::Data::new(config::get_identity_config());

        let token = identity_config.generate_identity(&user);

        let app = test::init_service(
            App::new()
                .app_data(pool)
                .app_data(identity_config)
                .service(me),
        )
        .await;

        let req = test::TestRequest::get()
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .uri("/me/")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: UserWithAuthProviders = test::read_body_json(resp).await;

        assert_eq!(
            body,
            UserWithAuthProviders {
                user,
                providers: vec![main_provider],
            }
        );
    }

    #[actix_web::test]
    async fn test_me_returns_providers_in_the_right_order() {
        lazy_static::lazy_static! {
            static ref USER_INFO: GoogleUserInfo = GoogleUserInfo {
                id: "0001".to_string(),
                email: Some("example@existing.com".to_string()),
                verified_email: true,
                name: Some("Adam".to_string()),
                family_name: None,
                given_name: Some("Adam".to_string()),
                locale: None,
                picture: None,
            };

            static ref ALT_USER_INFO: GoogleUserInfo = GoogleUserInfo {
                id: "0002".to_string(),
                email: Some("alt@existing.com".to_string()),
                verified_email: true,
                name: Some("Adam".to_string()),
                family_name: None,
                given_name: Some("Adam".to_string()),
                locale: None,
                picture: None,
            };
        }

        let user = User {
            id: Uuid::new_v4(),
            name: Some("Adam".to_string()),
        };

        let main_provider = USER_INFO.into_provider_insert(&user).into_row();
        let alt_provider = AuthProvider {
            order: 1,
            ..ALT_USER_INFO.into_provider_insert(&user).into_row()
        };

        let pool = db::initialize_db_pool(&config::get_db_url()).await;
        {
            let mut conn = pool
                .get()
                .await
                .expect("Failed to get a database connection");

            diesel::insert_into(schema::user::table)
                .values(&user)
                .execute(&mut conn)
                .await
                .expect("Failed to insert user");

            diesel::insert_into(schema::auth_provider::table)
                .values(&vec![alt_provider.clone(), main_provider.clone()])
                .execute(&mut conn)
                .await
                .expect("Failed to insert provider");
        }

        let pool = web::Data::new(pool);
        let identity_config = web::Data::new(config::get_identity_config());

        let token = identity_config.generate_identity(&user);

        let app = test::init_service(
            App::new()
                .app_data(pool)
                .app_data(identity_config)
                .service(me),
        )
        .await;

        let req = test::TestRequest::get()
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .uri("/me/")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: UserWithAuthProviders = test::read_body_json(resp).await;

        assert_eq!(
            body,
            UserWithAuthProviders {
                user,
                providers: vec![main_provider, alt_provider],
            }
        );
    }
}
