use diesel::{r2d2, PgConnection};

pub mod oauth2;
pub mod schema;
pub mod user;

/// Short-hand for the database pool type to use throughout the app.
pub type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;

pub type DbError = Box<dyn std::error::Error + Send + Sync>;

/// Given a Diesel struct `Data`, create a struct `InsertData` that contains
/// all the same fields except `id: Uuid`.
#[macro_export]
macro_rules! diesel_insertable {
    (
        #[derive($($path:path),+)]
        $(#[$attr:meta])*
        $vis:vis struct $Name:ident {
          $($field_vis:vis $field:ident : $FieldType:ty),* $(,)?
        }
    ) => {
        ::paste::paste! {
            #[derive($($path),+)]
            $(#[$attr])*
            $vis struct $Name {
                pub id: Uuid,
                $($field_vis $field : $FieldType), *
            }
            #[derive(::diesel::prelude::Insertable)]
            $(#[$attr])*
            $vis struct [<Insert $Name>] {
                $($field_vis $field : $FieldType), *
            }
            impl From<[<Insert $Name>]> for $Name {
                fn from(value: [<Insert $Name>]) -> Self {
                    Self {
                        id: Uuid::new_v4(),
                        $($field: value.$field),*
                    }
                }
            }
            impl From<$Name> for [<Insert $Name>] {
                fn from(value: $Name) -> Self {
                    Self {
                        $($field: value.$field),*
                    }
                }
            }
        }
    }
}
