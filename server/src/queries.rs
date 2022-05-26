use diesel::prelude::*;

use crate::models;
use crate::schema;

type DbError = Box<dyn std::error::Error + Send + Sync>;

pub fn get_player(conn: &SqliteConnection, itsf_lic: &str) -> Result<Option<models::Player>, DbError> {
    use crate::schema::players::dsl::*;

    let player = players
        .filter(itsf_license.eq(itsf_lic))
        .first::<models::Player>(conn)
        .optional()?;

    Ok(player)
}
