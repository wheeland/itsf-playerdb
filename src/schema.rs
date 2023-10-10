// @generated automatically by Diesel CLI.

diesel::table! {
    players (itsf_id) {
        itsf_id -> Integer,
        json_data -> Binary,
    }
}
