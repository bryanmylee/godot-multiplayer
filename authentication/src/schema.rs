// @generated automatically by Diesel CLI.

diesel::table! {
    auth_provider (id) {
        id -> Uuid,
        user_id -> Uuid,
        order -> Int2,
        provider_type -> Text,
        provider_id -> Text,
        email -> Nullable<Text>,
        email_verified -> Bool,
        display_name -> Nullable<Text>,
        user_name -> Nullable<Text>,
        picture_url -> Nullable<Text>,
        locale -> Nullable<Text>,
    }
}

diesel::table! {
    refresh_token (id) {
        id -> Uuid,
        user_id -> Uuid,
        value -> Text,
        issued_at -> Timestamptz,
        expires_at -> Timestamptz,
        count -> Int8,
        invalidated -> Bool,
    }
}

diesel::table! {
    user (id) {
        id -> Uuid,
        name -> Nullable<Text>,
    }
}

diesel::joinable!(auth_provider -> user (user_id));
diesel::joinable!(refresh_token -> user (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    auth_provider,
    refresh_token,
    user,
);
