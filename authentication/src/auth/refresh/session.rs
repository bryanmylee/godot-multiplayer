use crate::auth::identity::{Identity, IdentityConfig};
use crate::auth::Token;
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
use serde::{Deserialize, Serialize};
use uuid::Uuid;

diesel_insertable! {
    #[derive(Queryable, Selectable, Insertable, AsChangeset)]
    #[diesel(belongs_to(User))]
    #[diesel(table_name = schema::refresh_session)]
    #[diesel(check_for_backend(Pg))]
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct RefreshSession {
        pub user_id: Uuid,
        pub issued_at: DateTime<Utc>,
        pub expires_at: DateTime<Utc>,
        pub count: i64,
        pub invalidated: bool,
    }
}

impl RefreshSession {
    pub async fn create(
        conn: &mut DbConnection,
        config: &IdentityConfig,
        user_id: &Uuid,
    ) -> Result<Self, DbError> {
        let issued_at = Utc::now();
        let expires_at = issued_at + config.refresh_expires_in;
        let token_insert = RefreshSessionInsert {
            user_id: user_id.clone(),
            issued_at,
            expires_at,
            count: 0,
            invalidated: false,
        };
        let token: RefreshSession = diesel::insert_into(schema::refresh_session::table)
            .values(token_insert)
            .on_conflict(schema::refresh_session::user_id)
            .do_update()
            .set((
                schema::refresh_session::issued_at.eq(issued_at),
                schema::refresh_session::expires_at.eq(expires_at),
                schema::refresh_session::count.eq(schema::refresh_session::count + 1),
                schema::refresh_session::invalidated.eq(false),
            ))
            .get_result(conn)
            .await?;
        Ok(token)
    }

    pub fn generate_token(&self, config: &IdentityConfig) -> Token {
        Token {
            value: RefreshTokenClaims::from(self).encode(config),
            expires_at: self.expires_at,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum RefreshResult {
    Success(RefreshSuccess),
    TokenDecodeFailure,
    SessionNotFound,
    TokenAlreadyUsed,
    SessionExpired,
    SessionInvalidated,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct RefreshSuccess {
    pub access_token: Token,
    pub refresh_token: Token,
}

impl RefreshSession {
    pub async fn refresh(
        conn: &mut DbConnection,
        config: &IdentityConfig,
        refresh_token: &str,
    ) -> Result<RefreshResult, DbError> {
        let Ok(claims) = RefreshTokenClaims::decode(config, refresh_token) else {
            return Ok(RefreshResult::TokenDecodeFailure);
        };

        let Some(session): Option<RefreshSession> = schema::refresh_session::table
            .filter(schema::refresh_session::user_id.eq(&claims.sub))
            .first(conn)
            .await
            .optional()?
        else {
            return Ok(RefreshResult::SessionNotFound);
        };

        if claims.cnt < session.count {
            return RefreshSession::invalidate_session(conn, claims.sub)
                .await
                .and(Ok(RefreshResult::TokenAlreadyUsed));
        }

        if session.expires_at < Utc::now() {
            return RefreshSession::invalidate_session(conn, claims.sub)
                .await
                .and(Ok(RefreshResult::SessionExpired));
        }

        if session.invalidated {
            return Ok(RefreshResult::SessionInvalidated);
        }

        let access_token = Identity::from_user_id(&session.user_id).generate_token(config);
        let refresh_session = RefreshSession::create(conn, config, &claims.sub).await?;
        let refresh_token = refresh_session.generate_token(config);

        return Ok(RefreshResult::Success(RefreshSuccess {
            access_token,
            refresh_token,
        }));
    }

    async fn invalidate_session(conn: &mut DbConnection, user_id: Uuid) -> Result<(), DbError> {
        diesel::update(schema::refresh_session::table)
            .filter(schema::refresh_session::user_id.eq(&user_id))
            .set(schema::refresh_session::invalidated.eq(true))
            .execute(conn)
            .await
            .map(|_| ())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RefreshTokenClaims {
    pub sub: Uuid,
    pub iat: u64,
    pub exp: u64,
    pub cnt: i64,
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
        let mut validation = Validation::default();
        validation.validate_exp = false;
        match jsonwebtoken::decode::<Self>(
            token,
            &DecodingKey::from_secret(config.refresh_secret.as_ref()),
            &validation,
        ) {
            Ok(payload) => Ok(payload.claims),
            Err(err) => Err(error::ErrorBadRequest(err)),
        }
    }
}

impl From<&RefreshSession> for RefreshTokenClaims {
    fn from(value: &RefreshSession) -> Self {
        Self {
            sub: value.user_id,
            iat: value.issued_at.timestamp() as u64,
            exp: value.expires_at.timestamp() as u64,
            cnt: value.count,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{config, db, user::User};
    use chrono::Duration;

    use super::*;

    mod create {
        use super::*;

        #[actix_web::test]
        async fn create_inserts_session_to_db() {
            let user = User {
                id: Uuid::new_v4(),
                name: None,
            };

            let pool = db::initialize_db_pool(&config::DB_URL).await;
            let mut conn = pool
                .get()
                .await
                .expect("Failed to get a database connection");

            diesel::insert_into(schema::user::table)
                .values(&user)
                .execute(&mut conn)
                .await
                .expect("Failed to insert user");

            let new_session = RefreshSession::create(&mut conn, &config::IDENTITY_CONFIG, &user.id)
                .await
                .expect("Failed to create new token");

            assert_eq!(new_session.user_id, user.id);
            assert!(
                (Utc::now() - Duration::days(1)..Utc::now() + Duration::days(1))
                    .contains(&new_session.issued_at)
            );
            assert!(
                (Utc::now() + Duration::days(6)..Utc::now() + Duration::days(8))
                    .contains(&new_session.expires_at)
            );
            assert_eq!(new_session.count, 0);
            assert_eq!(new_session.invalidated, false);

            let stored_session: RefreshSession = schema::refresh_session::table
                .find(new_session.id)
                .first(&mut conn)
                .await
                .expect("Failed to find new session");
            assert_eq!(stored_session, new_session);
        }

        #[actix_web::test]
        async fn create_on_existing_session_updates_fields() {
            let user = User {
                id: Uuid::new_v4(),
                name: None,
            };

            let old_session = {
                let issued_at = Utc::now() - Duration::days(14);
                let expires_at = issued_at + Duration::days(7);

                RefreshSession {
                    id: Uuid::new_v4(),
                    user_id: user.id,
                    issued_at,
                    expires_at,
                    count: 5,
                    invalidated: true,
                }
            };

            let pool = db::initialize_db_pool(&config::DB_URL).await;
            let mut conn = pool
                .get()
                .await
                .expect("Failed to get a database connection");

            diesel::insert_into(schema::user::table)
                .values(&user)
                .execute(&mut conn)
                .await
                .expect("Failed to insert user");

            diesel::insert_into(schema::refresh_session::table)
                .values(&old_session)
                .execute(&mut conn)
                .await
                .expect("Failed to insert old token");

            let new_session = RefreshSession::create(&mut conn, &config::IDENTITY_CONFIG, &user.id)
                .await
                .expect("Failed to create new token");

            assert_eq!(new_session.user_id, user.id);
            assert!(
                (Utc::now() - Duration::days(1)..Utc::now() + Duration::days(1))
                    .contains(&new_session.issued_at)
            );
            assert!(
                (Utc::now() + Duration::days(6)..Utc::now() + Duration::days(8))
                    .contains(&new_session.expires_at)
            );
            assert_eq!(new_session.count, 6);
            assert_eq!(new_session.invalidated, false);

            let stored_session: RefreshSession = schema::refresh_session::table
                .find(new_session.id)
                .first(&mut conn)
                .await
                .expect("Failed to find new session");
            assert_eq!(stored_session, new_session);
        }
    }

    mod refresh {
        use super::*;

        #[actix_web::test]
        async fn refresh_creates_new_access_and_refresh_tokens() {
            let user = User {
                id: Uuid::new_v4(),
                name: None,
            };

            let session = {
                let issued_at = Utc::now();
                let expires_at = issued_at + Duration::days(7);

                RefreshSession {
                    id: Uuid::new_v4(),
                    user_id: user.id,
                    issued_at,
                    expires_at,
                    count: 5,
                    invalidated: false,
                }
            };

            let identity_config = &config::IDENTITY_CONFIG;

            let pool = db::initialize_db_pool(&config::DB_URL).await;
            let mut conn = pool
                .get()
                .await
                .expect("Failed to get a database connection");

            let refresh_token = {
                diesel::insert_into(schema::user::table)
                    .values(&user)
                    .execute(&mut conn)
                    .await
                    .expect("Failed to insert user");

                diesel::insert_into(schema::refresh_session::table)
                    .values(&session)
                    .execute(&mut conn)
                    .await
                    .expect("Failed to insert token");

                session.generate_token(identity_config)
            };

            let refresh_result =
                RefreshSession::refresh(&mut conn, &identity_config, &refresh_token.value)
                    .await
                    .expect("Failed to refresh session");

            assert!(matches!(refresh_result, RefreshResult::Success(_)));
            let RefreshResult::Success(success) = refresh_result else {
                panic!();
            };
            assert_ne!(success.refresh_token, refresh_token);
        }

        #[actix_web::test]
        async fn refresh_with_non_existent_token_errors() {
            let user = User {
                id: Uuid::new_v4(),
                name: None,
            };

            let session = {
                let issued_at = Utc::now();
                let expires_at = issued_at + Duration::days(7);

                RefreshSession {
                    id: Uuid::new_v4(),
                    user_id: user.id,
                    issued_at,
                    expires_at,
                    count: 5,
                    invalidated: false,
                }
            };

            let identity_config = &config::IDENTITY_CONFIG;

            let pool = db::initialize_db_pool(&config::DB_URL).await;
            let mut conn = pool
                .get()
                .await
                .expect("Failed to get a database connection");

            let refresh_token = {
                diesel::insert_into(schema::user::table)
                    .values(&user)
                    .execute(&mut conn)
                    .await
                    .expect("Failed to insert user");

                session.generate_token(identity_config)
            };

            let refresh_result =
                RefreshSession::refresh(&mut conn, &identity_config, &refresh_token.value)
                    .await
                    .expect("Failed to refresh session");

            assert_eq!(refresh_result, RefreshResult::SessionNotFound);
        }

        #[actix_web::test]
        async fn refresh_with_used_token_invalidates_existing_token() {
            let user = User {
                id: Uuid::new_v4(),
                name: None,
            };

            let session = {
                let issued_at = Utc::now();
                let expires_at = issued_at + Duration::days(7);

                RefreshSession {
                    id: Uuid::new_v4(),
                    user_id: user.id,
                    issued_at,
                    expires_at,
                    count: 5,
                    invalidated: false,
                }
            };

            let identity_config = &config::IDENTITY_CONFIG;

            let pool = db::initialize_db_pool(&config::DB_URL).await;
            let mut conn = pool
                .get()
                .await
                .expect("Failed to get a database connection");

            let refresh_token = {
                diesel::insert_into(schema::user::table)
                    .values(&user)
                    .execute(&mut conn)
                    .await
                    .expect("Failed to insert user");

                diesel::insert_into(schema::refresh_session::table)
                    .values(&session)
                    .execute(&mut conn)
                    .await
                    .expect("Failed to insert token");

                session.generate_token(identity_config)
            };

            let _ = RefreshSession::refresh(&mut conn, &identity_config, &refresh_token.value)
                .await
                .expect("Failed to refresh session");

            let refresh_result =
                RefreshSession::refresh(&mut conn, &identity_config, &refresh_token.value)
                    .await
                    .expect("Failed to refresh session");

            assert_eq!(refresh_result, RefreshResult::TokenAlreadyUsed);

            let stored_session: RefreshSession = schema::refresh_session::table
                .find(session.id)
                .first(&mut conn)
                .await
                .expect("Failed to find session");
            assert_eq!(stored_session.invalidated, true);
        }

        #[actix_web::test]
        async fn refresh_with_expired_token_invalidates_existing_token() {
            let user = User {
                id: Uuid::new_v4(),
                name: None,
            };

            let session = {
                let issued_at = Utc::now() - Duration::days(14);
                let expires_at = issued_at + Duration::days(7);

                RefreshSession {
                    id: Uuid::new_v4(),
                    user_id: user.id,
                    issued_at,
                    expires_at,
                    count: 5,
                    invalidated: false,
                }
            };

            let identity_config = &config::IDENTITY_CONFIG;

            let pool = db::initialize_db_pool(&config::DB_URL).await;
            let mut conn = pool
                .get()
                .await
                .expect("Failed to get a database connection");

            let refresh_token = {
                diesel::insert_into(schema::user::table)
                    .values(&user)
                    .execute(&mut conn)
                    .await
                    .expect("Failed to insert user");

                diesel::insert_into(schema::refresh_session::table)
                    .values(&session)
                    .execute(&mut conn)
                    .await
                    .expect("Failed to insert token");

                session.generate_token(identity_config)
            };

            let refresh_result =
                RefreshSession::refresh(&mut conn, &identity_config, &refresh_token.value)
                    .await
                    .expect("Failed to refresh session");

            assert_eq!(refresh_result, RefreshResult::SessionExpired);

            let stored_session: RefreshSession = schema::refresh_session::table
                .find(session.id)
                .first(&mut conn)
                .await
                .expect("Failed to find session");
            assert_eq!(stored_session.invalidated, true);
        }

        #[actix_web::test]
        async fn refresh_with_invalidated_token_errors() {
            let user = User {
                id: Uuid::new_v4(),
                name: None,
            };

            let session = {
                let issued_at = Utc::now();
                let expires_at = issued_at + Duration::days(7);

                RefreshSession {
                    id: Uuid::new_v4(),
                    user_id: user.id,
                    issued_at,
                    expires_at,
                    count: 5,
                    invalidated: true,
                }
            };

            let identity_config = &config::IDENTITY_CONFIG;

            let pool = db::initialize_db_pool(&config::DB_URL).await;
            let mut conn = pool
                .get()
                .await
                .expect("Failed to get a database connection");

            let refresh_token = {
                diesel::insert_into(schema::user::table)
                    .values(&user)
                    .execute(&mut conn)
                    .await
                    .expect("Failed to insert user");

                diesel::insert_into(schema::refresh_session::table)
                    .values(&session)
                    .execute(&mut conn)
                    .await
                    .expect("Failed to insert token");

                session.generate_token(identity_config)
            };

            let refresh_result =
                RefreshSession::refresh(&mut conn, &identity_config, &refresh_token.value)
                    .await
                    .expect("Failed to refresh session");

            assert_eq!(refresh_result, RefreshResult::SessionInvalidated);
        }
    }
}
