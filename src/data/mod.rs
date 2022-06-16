use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, Mutex},
};

mod db;
pub mod dtfb;
pub mod itsf;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Player {
    pub itsf_id: i32,

    pub first_name: String,
    pub last_name: String,
    pub birth_year: i32,
    pub country_code: Option<String>,
    pub category: itsf::PlayerCategory,

    pub itsf_rankings: Vec<itsf::Ranking>,

    pub dtfb_id: Option<i32>,
    pub dtfb_national_rankings: Vec<dtfb::NationalRanking>,
    pub dtfb_championship_results: Vec<dtfb::NationalChampionshipResult>,
    pub dtfb_league_teams: Vec<(i32, String)>,
}

pub struct PlayerImage {
    pub itsf_id: i32,
    pub image_data: Vec<u8>,
    pub image_format: String,
}

struct DatabaseInner {
    db: RefCell<db::DbConnection>,
    players: HashMap<i32, Player>,
}

#[derive(Clone)]
pub struct DatabaseRef {
    inner: Arc<Mutex<DatabaseInner>>,
}

impl DatabaseRef {
    pub fn load(path: &str) -> Self {
        let mut db = db::DbConnection::open(path);
        let mut players = HashMap::new();

        for player_id in db.get_player_ids() {
            let player = db
                .read_player_json(player_id)
                .expect("failed to read player");
            players.insert(player_id, player);
        }

        let inner = DatabaseInner {
            db: RefCell::new(db),
            players,
        };

        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    pub fn get_player(&self, itsf_id: i32) -> Option<Player> {
        let inner = self.inner.lock().unwrap();
        inner.players.get(&itsf_id).map(|player| player.clone())
    }

    pub fn add_player(&self, player: Player) {
        let mut inner = self.inner.lock().unwrap();
        inner
            .db
            .borrow_mut()
            .write_player_json(player.itsf_id, &player);
        inner.players.insert(player.itsf_id, player);
    }

    pub fn get_player_image(&self, itsf_id: i32) -> Option<PlayerImage> {
        let inner = self.inner.lock().unwrap();
        let mut db = inner.db.borrow_mut();
        db.read_player_image(itsf_id)
    }

    pub fn set_player_image(&self, player_image: PlayerImage) {
        let inner = self.inner.lock().unwrap();
        inner.db.borrow_mut().write_player_image(&player_image);
    }

    fn modify_player<F>(&self, itsf_id: i32, f: F)
    where
        F: FnOnce(&mut Player),
    {
        let mut inner = self.inner.lock().unwrap();

        if let Some(player) = inner.players.get_mut(&itsf_id) {
            f(player);
        }

        if let Some(player) = inner.players.get(&itsf_id) {
            inner.db.borrow_mut().write_player_json(itsf_id, &player);
        }
    }

    pub fn add_player_itsf_ranking(&self, itsf_id: i32, ranking: itsf::Ranking) {
        self.modify_player(itsf_id, |player| {
            player.itsf_rankings.retain(|r| !ranking.matches(&r));
            player.itsf_rankings.push(ranking);
        });
    }

    pub fn set_player_dtfb_id(&self, itsf_id: i32, dtfb_id: i32) {
        self.modify_player(itsf_id, |player| {
            player.dtfb_id = Some(dtfb_id);
        });
    }

    pub fn add_player_dtfb_championship_result(
        &self,
        itsf_id: i32,
        result: dtfb::NationalChampionshipResult,
    ) {
        self.modify_player(itsf_id, |player| {
            player
                .dtfb_championship_results
                .retain(|r| !result.matches(&r));
            player.dtfb_championship_results.push(result);
        });
    }

    pub fn add_player_dtfb_ranking(&self, itsf_id: i32, ranking: dtfb::NationalRanking) {
        self.modify_player(itsf_id, |player| {
            player
                .dtfb_national_rankings
                .retain(|r| !ranking.matches(&r));
            player.dtfb_national_rankings.push(ranking);
        });
    }

    pub fn add_player_dtfb_team(&self, itsf_id: i32, year: i32, team: String) {
        self.modify_player(itsf_id, |player| {
            player.dtfb_league_teams.retain(|t| t.0 != year);
            player.dtfb_league_teams.push((year, team));
        });
    }
}
