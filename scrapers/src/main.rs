use std::collections::HashMap;

use itsf_rankings::{ItsfRanking, get_itsf_rankings};

mod download;
mod itsf_rankings;

#[tokio::main]
async fn main() {
    let mut players = HashMap::new();

    for year in [2022, 2020, 2019, 2018] {
        for ranking in [ItsfRanking::Combined, ItsfRanking::Doubles, ItsfRanking::Singles] {
            match get_itsf_rankings(year, ranking).await {
                Ok(rankings) => {
                    println!("ranking for {} {:?}: {} players", year, ranking, rankings.len());

                    for placement in rankings {
                        *players.entry(placement.lic).or_insert(0) += 1000 - placement.place;
                    }
                },
                Err(err) => {
                    println!("error for {} {:?}: {} players", year, ranking, err);
                }
            }
        }
    }

    let mut players = players.iter().collect::<Vec<(&String, &u32)>>();
    players.sort_by(|a, b| { b.1.cmp(a.1) });

    println!("{} players total", players.len());
    for i in 0..100 {
        println!("{}: {}", players[i].1, players[i].0)
    }
}
