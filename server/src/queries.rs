use diesel::prelude::*;

use crate::models;

fn expect_result<T>(result: Result<T, diesel::result::Error>) -> T {
    match result {
        Ok(value) => value,
        Err(err) => panic!("SQL Error: {:?}", err),
    }
}

pub fn get_player(conn: &SqliteConnection, itsf_lic: &str) -> Option<models::Player> {
    use crate::schema::players::dsl::*;

    let player = players
        .filter(itsf_license.eq(itsf_lic))
        .first::<models::Player>(conn)
        .optional();

    expect_result(player)
}
