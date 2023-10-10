use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, Mutex},
};
use zip::{CompressionMethod, ZipWriter};

mod db;
pub mod dtfb;
pub mod itsf;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayerComment {
    pub timestamp: u32,
    pub text: String,
}

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
    pub dtfb_league_teams: Vec<dtfb::NationalTeam>,

    #[serde(default)]
    pub comments: Vec<PlayerComment>,
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
    database_path: String,
    image_directory: String,
    inner: Arc<Mutex<DatabaseInner>>,
}

fn add_zip_file(
    writer: &mut ZipWriter<Cursor<&mut Vec<u8>>>,
    compression: CompressionMethod,
    path: &str,
) -> Result<(), ()> {
    let mut f = File::open(path).map_err(|_| ())?;
    let mut data = Vec::new();
    f.read_to_end(&mut data).map_err(|_| ())?;

    let options = zip::write::FileOptions::default().compression_method(compression);
    // writer.start_file(path.split("/").last().unwrap(), options).map_err(|_| ())?;
    writer.start_file(path, options).map_err(|_| ())?;
    writer.write(&data).map_err(|_| ())?;

    Ok(())
}

impl DatabaseRef {
    pub fn load(path: &str, image_directory: &str) -> Self {
        let mut db = db::DbConnection::open(path);
        let mut players = HashMap::new();

        for player_id in db.get_player_ids() {
            let player = db.read_player_json(player_id).expect("failed to read player");
            players.insert(player_id, player);
        }
        log::error!("Loaded {} players", players.len());

        let inner = DatabaseInner {
            db: RefCell::new(db),
            players,
        };

        let path_info = std::fs::metadata(image_directory).unwrap_or_else(|_| panic!("Can't open {}", image_directory));
        assert!(path_info.is_dir(), "Not a directory: {}", image_directory);

        Self {
            inner: Arc::new(Mutex::new(inner)),
            image_directory: String::from(image_directory),
            database_path: String::from(path),
        }
    }

    pub fn get_player(&self, itsf_id: i32) -> Option<Player> {
        let inner = self.inner.lock().unwrap();
        inner.players.get(&itsf_id).cloned()
    }

    pub fn get_player_ids(&self) -> Vec<i32> {
        let inner = self.inner.lock().unwrap();
        inner.players.keys().copied().collect()
    }

    pub fn add_player(&self, player: Player) {
        let mut inner = self.inner.lock().unwrap();
        inner.db.borrow_mut().write_player_json(player.itsf_id, &player);
        inner.players.insert(player.itsf_id, player);
    }

    pub fn get_player_image(&self, itsf_id: i32) -> Option<PlayerImage> {
        let path = format!("{}/{}.jpg", self.image_directory, itsf_id);
        std::fs::read(path).ok().map(|image_data| PlayerImage {
            itsf_id,
            image_data,
            image_format: String::from("jpg"),
        })
    }

    pub fn set_player_image(&self, player_image: PlayerImage) {
        let path = format!("{}/{}.jpg", self.image_directory, player_image.itsf_id);
        std::fs::write(&path, player_image.image_data).unwrap_or_else(|_| panic!("Failed to write {}", path));
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
            player.itsf_rankings.retain(|r| !ranking.matches(r));
            player.itsf_rankings.push(ranking);
        });
    }

    pub fn set_player_dtfb_id(&self, itsf_id: i32, dtfb_id: i32) {
        self.modify_player(itsf_id, |player| {
            player.dtfb_id = Some(dtfb_id);
        });
    }

    pub fn add_player_dtfb_championship_result(&self, itsf_id: i32, result: dtfb::NationalChampionshipResult) {
        self.modify_player(itsf_id, |player| {
            player.dtfb_championship_results.retain(|r| !result.matches(r));
            player.dtfb_championship_results.push(result);
        });
    }

    pub fn add_player_dtfb_ranking(&self, itsf_id: i32, ranking: dtfb::NationalRanking) {
        self.modify_player(itsf_id, |player| {
            player.dtfb_national_rankings.retain(|r| !ranking.matches(r));
            player.dtfb_national_rankings.push(ranking);
        });
    }

    pub fn add_player_dtfb_team(&self, itsf_id: i32, year: i32, name: String) {
        self.modify_player(itsf_id, |player| {
            player.dtfb_league_teams.retain(|t| t.year != year);
            player.dtfb_league_teams.push(dtfb::NationalTeam { year, name });
        });
    }

    pub fn add_player_comment(&self, itsf_id: i32, text: String) {
        self.modify_player(itsf_id, |player| {
            let timestamp = chrono::Utc::now().naive_local().timestamp() as u32;
            player.comments.push(PlayerComment { timestamp, text });
            player.comments.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        });
    }

    pub fn create_zip_file(&self) -> Result<Vec<u8>, ()> {
        let mut buffer = Vec::new();
        {
            let mut zip = ZipWriter::new(Cursor::new(&mut buffer));
            add_zip_file(&mut zip, CompressionMethod::Deflated, &self.database_path)?;

            let options = zip::write::FileOptions::default().compression_method(CompressionMethod::Stored);
            zip.add_directory("images", options).map_err(|_| ())?;

            let dir = std::fs::read_dir(&self.image_directory).map_err(|_| ())?;
            for file in dir {
                let file = file.map_err(|_| ())?.path();
                let file = file.to_str().ok_or(())?;
                add_zip_file(&mut zip, CompressionMethod::Deflated, file)?;
            }
        }

        Ok(buffer)
    }
}
