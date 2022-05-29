use std::sync::{Arc, Weak};

use crate::{
    background::BackgroundOperationProgress,
    models::{ItsfRankingCategory, ItsfRankingClass},
    queries,
};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};
use diesel::{prelude::*, r2d2::ConnectionManager};
use r2d2::PooledConnection;

mod download;
pub mod itsf_rankings;
pub mod players;

async fn do_itsf_rankings_download(
    conn: PooledConnection<ConnectionManager<SqliteConnection>>,
    year: i32,
    category: ItsfRankingCategory,
    class: ItsfRankingClass,
    progress: Arc<BackgroundOperationProgress>,
) -> Result<(), String> {
    log::error!(
        "starting download of ITSF rankings for {}, {:?}, {:?}",
        year,
        category,
        class
    );
    let rankings = itsf_rankings::download(year, category, class, 100).await?;

    let missing_players: Vec<i32> = rankings
        .iter()
        .filter_map(|entry| match queries::get_player(&conn, entry.1) {
            None => Some(entry.1),
            Some(_) => None,
        })
        .collect();

    progress.log(format!("Downloaded player rankings"));
    progress.log(format!(
        "Downloading {} player info pages...",
        missing_players.len()
    ));
    progress.set_progress(1, missing_players.len() + 1);

    for missing_player in missing_players.iter().enumerate() {
        let player = players::download_player_info(*missing_player.1).await?;
        progress.log(format!(
            "Downloaded player info for {}, {}",
            player.last_name, player.first_name
        ));
        queries::add_player(&conn, player);
        progress.set_progress(missing_player.0 + 1, missing_players.len() + 1);
    }

    let queried_at = Utc::now().naive_utc();
    queries::add_itsf_rankings(&conn, year, queried_at, category, class, &rankings);

    Ok(())
}

pub fn start_itsf_rankings_download(
    conn: PooledConnection<ConnectionManager<SqliteConnection>>,
    year: i32,
    category: ItsfRankingCategory,
    class: ItsfRankingClass,
) -> Weak<BackgroundOperationProgress> {
    let (arc, weak) = BackgroundOperationProgress::new("ITSF Rankings Download", 1);
    tokio::spawn(async move {
        match do_itsf_rankings_download(conn, year, category, class, arc).await {
            Ok(_) => {}
            Err(err) => log::error!("failed to download ITSF rankings: {}", err),
        }
    });
    weak
}
