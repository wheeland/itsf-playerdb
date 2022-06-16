table! {
    dtfb_national_championship_results (player_itsf_id, year) {
        player_itsf_id -> Integer,
        year -> Integer,
        place -> Integer,
        category -> Integer,
        class -> Integer,
    }
}

table! {
    dtfb_national_rankings (player_itsf_id, year) {
        player_itsf_id -> Integer,
        year -> Integer,
        place -> Integer,
        category -> Integer,
    }
}

table! {
    dtfb_player_ids (itsf_id) {
        itsf_id -> Integer,
        dtfb_id -> Integer,
    }
}

table! {
    dtfb_player_teams (player_itsf_id, year) {
        player_itsf_id -> Integer,
        year -> Integer,
        team_name -> Text,
    }
}

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
        birth_year -> Integer,
        country_code -> Nullable<Text>,
        category -> Integer,
    }
}

allow_tables_to_appear_in_same_query!(
    dtfb_national_championship_results,
    dtfb_national_rankings,
    dtfb_player_ids,
    dtfb_player_teams,
    itsf_ranking_entries,
    itsf_rankings,
    player_images,
    players,
);
