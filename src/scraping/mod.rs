use std::{
    collections::HashSet,
    sync::{Arc, Weak},
};

use crate::{
    background::BackgroundOperationProgress,
    data::DatabaseRef,
    data::{dtfb, itsf},
};
use futures_util::future::join_all;

mod download;
mod dtfb_players;
mod itsf_rankings;
mod players;

async fn download_itsf_players(
    db: &DatabaseRef,
    player_itsf_ids: &[i32],
    progress: Arc<BackgroundOperationProgress>,
) -> Result<(), String> {
    let mut missing_players: Vec<i32> = player_itsf_ids
        .into_iter()
        .filter_map(|itsf_lic| match db.get_player(*itsf_lic) {
            None => Some(*itsf_lic),
            Some(_) => None,
        })
        .collect();

    if missing_players.len() > 0 {
        progress.set_progress(1, missing_players.len() + 1);
        progress.log(format!(
            "[ITSF] Downloading {} ITSF player profiles",
            missing_players.len()
        ));

        // query players in sets of N, to hide ITSF server latency
        const MAX_CONCURRENT: usize = 5;
        while missing_players.len() > 0 {
            let mut player_futures = Vec::new();
            let mut image_futures = Vec::new();
            let count = missing_players.len().min(MAX_CONCURRENT);
            for _ in 0..count {
                let itsf_id = missing_players.pop().unwrap();
                player_futures.push(players::download_player_info(itsf_id));
                image_futures.push(players::download_player_image(itsf_id));
            }

            for player in join_all(player_futures).await {
                match player {
                    Ok(player) => {
                        progress.log(format!(
                            "[ITSF] .. downloaded player info for ID={}: {} {} ({:?}, {:?})",
                            player.itsf_id,
                            player.first_name,
                            player.last_name,
                            itsf::PlayerCategory::try_from(player.category).unwrap(),
                            player.country_code
                        ));
                        db.add_player(player);
                    }
                    Err(err) => {
                        progress.log(format!("[ITSF] Failed to download player: {}", err));
                    }
                }
            }

            for image in join_all(image_futures).await {
                if let Some(image) = image? {
                    db.set_player_image(image);
                }
            }
        }

        progress.log(format!("[ITSF] Done"));
    }

    Ok(())
}

async fn do_itsf_rankings_downloads(
    db: &DatabaseRef,
    years: Vec<i32>,
    categories: Vec<itsf::RankingCategory>,
    classes: Vec<itsf::RankingClass>,
    progress: Arc<BackgroundOperationProgress>,
    max_rank: usize,
) -> Result<(), String> {
    for year in years {
        for category in categories.iter().cloned() {
            for class in classes.iter().cloned() {
                progress.log(format!(
                    "[ITSF] Scraping ITSF rankings for {}, {:?}, {:?}",
                    year, category, class
                ));
                let rankings = itsf_rankings::download(year, category, class, max_rank).await?;

                let itsf_player_ids: Vec<i32> = rankings.iter().map(|entry| entry.1).collect();
                download_itsf_players(db, &itsf_player_ids, progress.clone()).await?;

                for placement in rankings {
                    db.add_player_itsf_ranking(
                        placement.1,
                        itsf::Ranking {
                            year,
                            category,
                            class,
                            place: placement.0,
                        },
                    );
                }
            }
        }
    }
    Ok(())
}

pub fn start_itsf_rankings_download(
    db: DatabaseRef,
    years: Vec<i32>,
    categories: Vec<itsf::RankingCategory>,
    classes: Vec<itsf::RankingClass>,
    max_rank: usize,
) -> Weak<BackgroundOperationProgress> {
    let (arc, weak) = BackgroundOperationProgress::new("ITSF Rankings Download", 1);
    tokio::spawn(async move {
        match do_itsf_rankings_downloads(&db, years, categories, classes, arc.clone(), max_rank).await {
            Ok(_) => {}
            Err(err) => log::error!("failed to download ITSF rankings: {}", err),
        };
        arc.set_progress(1, 1);
    });
    weak
}

async fn do_dtfb_rankings_download(
    db: DatabaseRef,
    seasons: Vec<i32>,
    progress: Arc<BackgroundOperationProgress>,
    max_rank: usize,
) -> Result<(), String> {
    progress.log(format!(
        "[DTFB] starting download of DTFB rankings for seasons {:?}",
        seasons
    ));

    let mut dtfb_player_ids = HashSet::new();

    for season in seasons {
        let ranking_ids = dtfb_players::collect_dtfb_rankings_for_season(season).await?;
        for ranking_id in ranking_ids {
            let rankings = dtfb_players::collect_dtfb_ids_from_rankings(ranking_id, max_rank).await?;
            for id in rankings {
                dtfb_player_ids.insert(id);
            }
        }
    }

    progress.log(format!("[DTFB] Downloading {} players", dtfb_player_ids.len()));

    let mut dtfb_player_ids: Vec<i32> = dtfb_player_ids.into_iter().collect();
    let mut dtfb_players = Vec::new();

    // download DTFB player profiles for every single player
    const MAX_CONCURRENT: usize = 5;
    while dtfb_player_ids.len() > 0 {
        let mut player_futures = Vec::new();
        let count = dtfb_player_ids.len().min(MAX_CONCURRENT);
        for _ in 0..count {
            let dtfb_id = dtfb_player_ids.pop().unwrap();
            player_futures.push(dtfb_players::DtfbPlayerInfo::download(dtfb_id));
        }

        for dtfb_player in join_all(player_futures).await {
            if let Ok(dtfb_player) = dtfb_player {
                progress.log(format!(
                    "[DTFB] .. downloaded player info for DTFB={}, ITSF={}",
                    dtfb_player.dtfb_id, dtfb_player.itsf_id,
                ));
                dtfb_players.push(dtfb_player);
            }
        }
    }

    let itsf_player_ids: Vec<i32> = dtfb_players.iter().map(|player| player.itsf_id).collect();
    download_itsf_players(&db, &itsf_player_ids, progress.clone()).await?;

    // add DTFB player data to DB
    for dtfb_player in dtfb_players {
        db.set_player_dtfb_id(dtfb_player.itsf_id, dtfb_player.dtfb_id);

        for result in dtfb_player.championship_results {
            db.add_player_dtfb_championship_result(
                dtfb_player.itsf_id,
                dtfb::NationalChampionshipResult {
                    year: result.year,
                    place: result.place,
                    category: result.category.into(),
                    class: result.class.into(),
                },
            );
        }

        for ranking in dtfb_player.national_rankings {
            db.add_player_dtfb_ranking(
                dtfb_player.itsf_id,
                dtfb::NationalRanking {
                    year: ranking.year,
                    place: ranking.place,
                    category: ranking.category.into(),
                },
            );
        }

        for team in dtfb_player.teams {
            db.add_player_dtfb_team(dtfb_player.itsf_id, team.0, team.1.clone());
        }
    }

    progress.log(format!("[DTFB] done"));

    Ok(())
}

pub fn start_dtfb_rankings_download(
    db: DatabaseRef,
    seasons: Vec<i32>,
    max_rank: usize,
) -> Weak<BackgroundOperationProgress> {
    let (arc, weak) = BackgroundOperationProgress::new("DTFB Rankings Download", 1);
    tokio::spawn(async move {
        match do_dtfb_rankings_download(db, seasons, arc.clone(), max_rank).await {
            Ok(_) => {}
            Err(err) => log::error!("failed to download DTFB rankings: {}", err),
        };
        arc.set_progress(1, 1);
    });
    weak
}
