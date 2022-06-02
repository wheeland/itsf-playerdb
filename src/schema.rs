table! {
    itsf_ranking_entries (itsf_ranking_id, place) {
        itsf_ranking_id -> Integer,
        place -> Integer,
        player_itsf_id -> Integer,
    }
}

table! {
    itsf_rankings (id) {
        id -> Integer,
        year -> Integer,
        queried_at -> Timestamp,
        count -> Integer,
        category -> Integer,
        class -> Integer,
    }
}

table! {
    player_images (itsf_id) {
        itsf_id -> Integer,
        image_data -> Binary,
        image_format -> Nullable<Text>,
    }
}

table! {
    players (itsf_id) {
        itsf_id -> Integer,
        first_name -> Text,
        last_name -> Text,
        dtfb_license -> Nullable<Text>,
        birth_year -> Integer,
        country_code -> Nullable<Text>,
        category -> Integer,
    }
}

allow_tables_to_appear_in_same_query!(
    itsf_ranking_entries,
    itsf_rankings,
    player_images,
    players,
);