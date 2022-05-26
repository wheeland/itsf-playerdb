table! {
    players (id) {
        id -> Integer,
        first_name -> Text,
        last_name -> Text,
        itsf_license -> Text,
        dtfb_license -> Nullable<Text>,
        birth_year -> Integer,
        country_code -> Nullable<Text>,
        category -> Nullable<Text>,
    }
}
