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
mod itsf_rankings;
mod players;
mod dtfb_players;

async fn download_itsf_players(
    conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    player_itsf_ids: &[i32],
    progress: Arc<BackgroundOperationProgress>,
) -> Result<(), String> {
    let mut missing_players: Vec<i32> = player_itsf_ids
        .into_iter()
        .filter_map(|itsf_lic| match queries::get_player(conn, *itsf_lic) {
            None => Some(*itsf_lic),
            Some(_) => None,
        })
        .collect();

    if missing_players.len() > 0 {
        progress.set_progress(1, missing_players.len() + 1);
        progress.log(format!("[ITSF] Downloading {} ITSF player profiles", missing_players.len()));

        // query players in sets of N, to hide ITSF server latency
        const MAX_CONCURRENT: usize = 10;
        while missing_players.len() > 0 {
            let mut player_futures = Vec::new();
            let mut image_futures = Vec::new();
            let count = missing_players.len().max(MAX_CONCURRENT);
            for _ in 0..count {
                let itsf_id = missing_players.pop().unwrap();
                player_futures.push(players::download_player_info(itsf_id));
                image_futures.push(players::download_player_image(itsf_id));
            }

            for player in join_all(player_futures).await {
                if let Ok(player) = player {
                    progress.log(format!(
                        "[ITSF] Downloaded player info for {}: {} {} ({:?}, {:?})",
                        player.itsf_id,
                        player.first_name,
                        player.last_name,
                        PlayerCategory::try_from(player.category).unwrap(),
                        player.country_code
                    ));
                    queries::add_player(conn, player);
                }
            }

            for image in join_all(image_futures).await {
                if let Some(image) = image? {
                    queries::add_player_image(conn, image);
                }
            }
        }
        
        progress.log(format!("[ITSF] ... done"));
    }

    Ok(())
}

async fn do_itsf_rankings_downloads(
    conn: PooledConnection<ConnectionManager<SqliteConnection>>,
    years: Vec<i32>,
    categories: Vec<ItsfRankingCategory>,
    classes: Vec<ItsfRankingClass>,
    progress: Arc<BackgroundOperationProgress>,
) -> Result<(), String> {
    for year in years {
        for category in &categories {
            for class in &classes {
                log::error!(
                    "starting download of ITSF rankings for {}, {:?}, {:?}",
                    year,
                    category,
                    class
                );
                let rankings = itsf_rankings::download(year, *category, *class, 100).await?;

                let itsf_player_ids: Vec<i32> = rankings.iter().map(|entry| entry.1).collect();
                download_itsf_players(&mut conn, &itsf_player_ids, progress.clone()).await?;

                let queried_at = Utc::now().naive_utc();
                queries::add_itsf_rankings(&conn, year, queried_at, *category, *class, &rankings);
            }
        }
    }
    Ok(())
}

pub fn start_itsf_rankings_download(
    conn: PooledConnection<ConnectionManager<SqliteConnection>>,
    years: Vec<i32>,
    categories: Vec<ItsfRankingCategory>,
    classes: Vec<ItsfRankingClass>,
) -> Weak<BackgroundOperationProgress> {
    let (arc, weak) = BackgroundOperationProgress::new("ITSF Rankings Download", 1);
    tokio::spawn(async move {
        match do_itsf_rankings_downloads(conn, years, categories, classes, arc.clone()).await {
            Ok(_) => {}
            Err(err) => log::error!("failed to download ITSF rankings: {}", err),
        };
        arc.set_progress(1, 1);
    });
    weak
}
