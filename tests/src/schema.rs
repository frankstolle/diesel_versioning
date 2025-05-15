// @generated automatically by Diesel CLI.

diesel::table! {
    simple (id) {
        id -> Integer,
        version -> Integer,
        body -> Text,
    }
}
