// @generated automatically by Diesel CLI.

diesel::table! {
    user (id) {
        id -> Uuid,
        email -> Nullable<Text>,
        email_verified -> Bool,
        locale -> Nullable<Text>,
        oauth2_id -> Nullable<Text>,
        oauth2_name -> Nullable<Text>,
        oauth2_picture_url -> Nullable<Text>,
    }
}
