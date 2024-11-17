// @generated automatically by Diesel CLI.

diesel::table! {
    history (id) {
        id -> Integer,
        word -> Text,
        created_at -> Timestamp,
    }
}
