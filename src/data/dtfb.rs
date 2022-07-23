#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(i8)]
pub enum ChampionshipCategory {
    #[serde(rename = "men")] Men,
    #[serde(rename = "women")] Women,
    #[serde(rename = "junior")] Junior,
    #[serde(rename = "senior")] Senior,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(i8)]
pub enum ChampionshipClass {
    #[serde(rename = "singles")] Singles,
    #[serde(rename = "doubles")] Doubles,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct NationalChampionshipResult {
    pub year: i32,
    pub place: i32,
    pub category: ChampionshipCategory,
    pub class: ChampionshipClass,
}

impl NationalChampionshipResult {
    pub fn matches(&self, other_ranking: &Self) -> bool {
        self.year == other_ranking.year && self.category == other_ranking.category && self.class == other_ranking.class
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct NationalRanking {
    pub year: i32,
    pub place: i32,
    pub category: ChampionshipCategory,
}

impl NationalRanking {
    pub fn matches(&self, other_ranking: &Self) -> bool {
        self.year == other_ranking.year && self.category == other_ranking.category
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NationalTeam {
    pub year: i32,
    pub name: String,
}
