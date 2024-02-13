use crate::auth::identity::IdentityConfig;
use crate::db::{DbConnection, DbError};
use crate::{diesel_insertable, schema};
use actix_web::error;
use chrono::{DateTime, Utc};
use diesel::{pg::Pg, prelude::*};
use diesel_async::RunQueryDsl;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::EncodingKey;
use jsonwebtoken::Header;
use jsonwebtoken::Validation;
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

diesel_insertable! {
    #[derive(Queryable, Selectable, Insertable, AsChangeset)]
    #[diesel(belongs_to(User))]
    #[diesel(table_name = schema::refresh_token)]
    #[diesel(check_for_backend(Pg))]
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct RefreshToken {
        pub user_id: Uuid,
        pub value: String,
        pub issued_at: DateTime<Utc>,
        pub expires_at: DateTime<Utc>,
        pub count: i64,
        pub invalidated: bool,
    }
}

impl RefreshToken {
    pub async fn create(
        conn: &mut DbConnection,
        user_id: &Uuid,
        config: &IdentityConfig,
    ) -> Result<Self, DbError> {
        let value = Alphanumeric.sample_string(&mut rand::thread_rng(), 64);
        let issued_at = Utc::now();
        let expires_at = issued_at + config.refresh_expires_in;
        let token_insert = RefreshTokenInsert {
            user_id: user_id.clone(),
            value: value.clone(),
            issued_at,
            expires_at,
            count: 0,
            invalidated: false,
        };
        let token: RefreshToken = diesel::insert_into(schema::refresh_token::table)
            .values(token_insert)
            .on_conflict(schema::refresh_token::user_id)
            .do_update()
            .set((
                schema::refresh_token::value.eq(value),
                schema::refresh_token::issued_at.eq(issued_at),
                schema::refresh_token::expires_at.eq(expires_at),
                schema::refresh_token::count.eq(schema::refresh_token::count + 1),
                schema::refresh_token::invalidated.eq(false),
            ))
            .get_result(conn)
            .await?;
        Ok(token)
    }

    pub fn generate_token(&self, config: &IdentityConfig) -> String {
        RefreshTokenClaims::from(self).encode(config)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RefreshTokenClaims {
    pub id: String,
    pub sub: String,
    pub iat: u64,
    pub exp: u64,
}

impl RefreshTokenClaims {
    pub fn encode(&self, config: &IdentityConfig) -> String {
        jsonwebtoken::encode(
            &Header::default(),
            &self,
            &EncodingKey::from_secret(config.refresh_secret.as_ref()),
        )
        .expect("Failed to generate refresh token")
    }

    pub fn decode(config: &IdentityConfig, token: &str) -> Result<Self, error::Error> {
        let Ok(payload) = jsonwebtoken::decode::<Self>(
            token,
            &DecodingKey::from_secret(config.refresh_secret.as_ref()),
            &Validation::default(),
        ) else {
            return Err(error::ErrorUnauthorized("Invalid refresh token"));
        };

        Ok(payload.claims)
    }
}

impl From<&RefreshToken> for RefreshTokenClaims {
    fn from(value: &RefreshToken) -> Self {
        Self {
            id: value.id.to_string(),
            sub: value.user_id.to_string(),
            iat: value.issued_at.timestamp() as u64,
            exp: value.expires_at.timestamp() as u64,
        }
    }
}

pub struct RefreshSuccess {
    access_token: String,
    refresh_token: String,
}

pub enum RefreshError {
    AlreadyUsed,
}

pub type RefreshResult = Result<RefreshSuccess, RefreshError>;

impl RefreshToken {
    pub async fn refresh(conn: &mut DbConnection, config: &IdentityConfig) -> RefreshResult {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod create {
        use chrono::Duration;

        use crate::{config, db, user::User};

        use super::*;

        #[actix_web::test]
        async fn test_create() {
            let user = User {
                id: Uuid::new_v4(),
                name: None,
            };

            let pool = db::initialize_db_pool(&config::get_db_url()).await;
            let mut conn = pool
                .get()
                .await
                .expect("Failed to get a database connection");

            diesel::insert_into(schema::user::table)
                .values(&user)
                .execute(&mut conn)
                .await
                .expect("Failed to insert user");

            let new_token =
                RefreshToken::create(&mut conn, &user.id, &config::get_identity_config())
                    .await
                    .expect("Failed to create new token");

            assert_eq!(new_token.user_id, user.id);
            assert!(
                (Utc::now() - Duration::days(1)..Utc::now() + Duration::days(1))
                    .contains(&new_token.issued_at)
            );
            assert!(
                (Utc::now() + Duration::days(6)..Utc::now() + Duration::days(8))
                    .contains(&new_token.expires_at)
            );
            assert_eq!(new_token.count, 0);
            assert_eq!(new_token.invalidated, false);
        }

        #[actix_web::test]
        async fn test_create_on_existing_refresh_token_for_user_updates_fields() {
            let user = User {
                id: Uuid::new_v4(),
                name: None,
            };
            let old_token = {
                let issued_at = Utc::now() - Duration::days(14);
                let expires_at = issued_at + Duration::days(7);

                RefreshToken {
                    id: Uuid::new_v4(),
                    value: "0000".to_string(),
                    user_id: user.id,
                    issued_at,
                    expires_at,
                    count: 5,
                    invalidated: true,
                }
            };

            let pool = db::initialize_db_pool(&config::get_db_url()).await;
            let mut conn = pool
                .get()
                .await
                .expect("Failed to get a database connection");

            diesel::insert_into(schema::user::table)
                .values(&user)
                .execute(&mut conn)
                .await
                .expect("Failed to insert user");

            diesel::insert_into(schema::refresh_token::table)
                .values(&old_token)
                .execute(&mut conn)
                .await
                .expect("Failed to insert old token");

            let new_token =
                RefreshToken::create(&mut conn, &user.id, &config::get_identity_config())
                    .await
                    .expect("Failed to create new token");

            assert_eq!(new_token.user_id, user.id);
            assert_ne!(new_token.value, old_token.value);
            assert!(
                (Utc::now() - Duration::days(1)..Utc::now() + Duration::days(1))
                    .contains(&new_token.issued_at)
            );
            assert!(
                (Utc::now() + Duration::days(6)..Utc::now() + Duration::days(8))
                    .contains(&new_token.expires_at)
            );
            assert_eq!(new_token.count, 6);
            assert_eq!(new_token.invalidated, false);
        }
    }
}
