table! {
    player_images (itsf_id) {
        itsf_id -> Integer,
        image_data -> Binary,
        image_format -> Text,
    }
}

table! {
    players (itsf_id) {
        itsf_id -> Integer,
        json_data -> Binary,
    }
}

allow_tables_to_appear_in_same_query!(player_images, players,);
