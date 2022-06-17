#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(i8)]
pub enum ChampionshipCategory {
    Men,
    Women,
    Junior,
    Senior,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(i8)]
pub enum ChampionshipClass {
    Singles,
    Doubles,
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
