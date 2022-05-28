table! {
    players (itsf_id) {
        itsf_id -> Integer,
        first_name -> Text,
        last_name -> Text,
        dtfb_license -> Nullable<Text>,
        birth_year -> Integer,
        country_code -> Nullable<Text>,
        category -> Nullable<Text>,
    }
}
