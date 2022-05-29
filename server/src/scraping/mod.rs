use std::sync::{Arc, Weak};

use crate::{
    background::BackgroundOperationProgress,
    models::{ItsfRankingCategory, ItsfRankingClass, PlayerCategory},
    queries,
};
use chrono::Utc;
use diesel::{prelude::*, r2d2::ConnectionManager};
use futures_util::future::join_all;
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

    let mut missing_players: Vec<i32> = rankings
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

    // query players in sets of N, to hide ITSF server latency
    const MAX_CONCURRENT: usize = 10;
    while missing_players.len() >= MAX_CONCURRENT {
        let mut player_futures = Vec::new();
        let mut image_futures = Vec::new();
        for _ in 0..MAX_CONCURRENT {
            let itsf_id = missing_players.pop().unwrap();
            player_futures.push(players::download_player_info(itsf_id));
            image_futures.push(players::download_player_image(itsf_id));
        }

        for player in join_all(player_futures).await {
            let player = player?;
            progress.log(format!(
                "Downloaded player info for {}: {} {} ({:?}, {:?})",
                player.itsf_id,
                player.first_name,
                player.last_name,
                PlayerCategory::try_from(player.category).unwrap(),
                player.country_code
            ));
            queries::add_player(&conn, player);
        }

        for image in join_all(image_futures).await {
            if let Some(image) = image? {
                queries::add_player_image(&conn, image);
            }
        }
    }

    let queried_at = Utc::now().naive_utc();
    queries::add_itsf_rankings(&conn, year, queried_at, category, class, &rankings);

    progress.set_progress(1, 1);

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
