mod me;

use crate::db::DbError;
use crate::{auth::provider::AuthProvider, db::DbConnection};
use crate::{diesel_insertable, schema};
use actix_web::web;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

diesel_insertable! {
    #[derive(Queryable, Selectable, Insertable, AsChangeset)]
    #[diesel(table_name = crate::schema::user)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct User {
        pub name: Option<String>,
    }
}

pub fn config_service(cfg: &mut web::ServiceConfig) {
    cfg.service(me::me);
}

impl Default for UserInsert {
    fn default() -> Self {
        UserInsert { name: None }
    }
}

impl User {
    pub async fn get_providers(
        &self,
        conn: &mut DbConnection,
    ) -> Result<Vec<AuthProvider>, DbError> {
        AuthProvider::belonging_to(self)
            .select(AuthProvider::as_select())
            .order(schema::auth_provider::order.asc())
            .load(conn)
            .await
    }
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(Deserialize, PartialEq))]
pub struct UserWithAuthProviders {
    #[serde(flatten)]
    pub user: User,
    pub providers: Vec<AuthProvider>,
}
