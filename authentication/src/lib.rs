pub mod auth;
pub mod config;
pub mod db;
pub mod schema;
pub mod user;

/// Given a Diesel struct `Data`, create a struct `InsertData` that contains
/// all the same fields except `id: Uuid`.
#[macro_export]
macro_rules! diesel_insertable {
    (
        #[derive($($path:path),+)]
        $(#[$attr:meta])*
        $vis:vis struct $Name:ident {}
    ) => {
            #[derive(::diesel::prelude::Identifiable)]
            #[derive($($path),+)]
            $(#[$attr])*
            $vis struct $Name {
                pub id: Uuid,
            }
    };
    (
        #[derive($($path:path),+)]
        $(#[$attr:meta])*
        $vis:vis struct $Name:ident {
          $($field_vis:vis $field:ident : $FieldType:ty),+ $(,)?
        }
    ) => {
        ::paste::paste! {
            #[derive(::diesel::prelude::Identifiable)]
            #[derive($($path),+)]
            $(#[$attr])*
            $vis struct $Name {
                pub id: Uuid,
                $($field_vis $field : $FieldType), *
            }

            #[derive(::diesel::prelude::Insertable)]
            $(#[$attr])*
            $vis struct [<$Name Insert>] {
                $($field_vis $field : $FieldType), *
            }

            impl [<$Name Insert>] {
                pub fn into_row(self) -> $Name {
                    $Name {
                        id: Uuid::new_v4(),
                        $($field: self.$field),*
                    }
                }
            }

            impl $Name {
                pub fn into_insert(self) -> [<$Name Insert>] {
                    [<$Name Insert>] {
                        $($field: self.$field),*
                    }
                }
            }
        }
    };
}
