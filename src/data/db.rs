use diesel::sqlite::SqliteConnection;
use diesel::{prelude::*, Insertable, Queryable};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::schema::*;

use super::PlayerImage;

#[derive(Queryable, Insertable, AsChangeset)]
#[table_name = "players"]
struct DbPlayer {
    itsf_id: i32,
    json_data: Vec<u8>,
}

#[derive(Queryable, Insertable, AsChangeset)]
#[table_name = "player_images"]
struct DbPlayerImage {
    itsf_id: i32,
    image_data: Vec<u8>,
    image_format: String,
}

pub struct DbConnection {
    conn: SqliteConnection,
}

fn expect_result<T>(result: Result<T, diesel::result::Error>) -> T {
    match result {
        Ok(value) => value,
        Err(err) => panic!("SQL Error: {:?}", err),
    }
}

impl DbConnection {
    pub fn open(path: &str) -> Self {
        let conn = SqliteConnection::establish(path).expect("Failed to open DB");
        Self { conn }
    }

    pub fn get_player_ids(&mut self) -> Vec<i32> {
        use crate::schema::players::dsl;

        let ids = dsl::players.select(dsl::itsf_id).load(&mut self.conn);

        expect_result(ids)
    }

    pub fn write_player_json<T: Serialize>(&mut self, itsf_id: i32, data: &T) {
        let json_data = serde_json::to_vec(&data).expect("JSON serialization failed");
        let player = DbPlayer { itsf_id, json_data };

        use crate::schema::players::dsl;

        let result = diesel::insert_into(dsl::players)
            .values(&player)
            .on_conflict(dsl::itsf_id)
            .do_update()
            .set(&player)
            .execute(&mut self.conn);

        let result = expect_result(result);
        if result != 1 {
            panic!("invalid query result for player insert: {}", result);
        }
    }

    pub fn read_player_json<T: DeserializeOwned>(&mut self, itsf_id: i32) -> Result<T, String> {
        use crate::schema::players::dsl;

        let player = dsl::players
            .filter(dsl::itsf_id.eq(itsf_id))
            .first::<DbPlayer>(&mut self.conn)
            .optional();

        match expect_result(player) {
            Some(player) => serde_json::from_slice(&player.json_data)
                .map_err(|err| format!("JSON Error when loading player {}: {}", itsf_id, err)),
            None => Err(format!("No player data found for player {}", itsf_id)),
        }
    }

    pub fn write_player_image(&mut self, image: &PlayerImage) {
        let player_image = DbPlayerImage {
            itsf_id: image.itsf_id,
            image_data: image.image_data.clone(),
            image_format: image.image_format.clone(),
        };

        use crate::schema::player_images::dsl;

        let result = diesel::insert_into(dsl::player_images)
            .values(&player_image)
            .on_conflict(dsl::itsf_id)
            .do_update()
            .set(&player_image)
            .execute(&mut self.conn);

        let result = expect_result(result);
        if result != 1 {
            panic!("invalid query result for player image insert: {}", result);
        }
    }

    pub fn read_player_image(&mut self, itsf_id: i32) -> Option<PlayerImage> {
        use crate::schema::player_images::dsl;

        let player_image = dsl::player_images
            .filter(dsl::itsf_id.eq(itsf_id))
            .first::<DbPlayerImage>(&mut self.conn)
            .optional();

        expect_result(player_image).map(|player_image| PlayerImage {
            itsf_id,
            image_data: player_image.image_data,
            image_format: player_image.image_format,
        })
    }
}
