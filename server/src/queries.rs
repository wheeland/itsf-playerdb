use diesel::prelude::*;

use crate::models;

fn expect_result<T>(result: Result<T, diesel::result::Error>) -> T {
    match result {
        Ok(value) => value,
        Err(err) => panic!("SQL Error: {:?}", err),
    }
}

pub fn get_player(conn: &SqliteConnection, itsf_lic: i32) -> Option<models::Player> {
    use crate::schema::players::dsl::*;

    let player = players
        .filter(itsf_id.eq(itsf_lic))
        .first::<models::Player>(conn)
        .optional();

    expect_result(player)
}

pub fn add_player(conn: &SqliteConnection, new_player: models::NewPlayer) -> bool {
    use crate::schema::players::dsl::*;

    let result = diesel::insert_or_ignore_into(players)
        .values(new_player)
        .execute(conn);

    match expect_result(result) {
        0 => false,
        1 => true,
        _ => panic!("invalid query result for player insert"),
    }
}
