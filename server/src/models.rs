use super::schema::*;

#[derive(Debug, Clone, Insertable, Queryable, serde::Serialize, serde::Deserialize)]
pub struct Player {
    pub itsf_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub dtfb_license: Option<String>,
    pub birth_year: i32,
    pub country_code: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Queryable, Insertable)]
#[table_name = "itsf_ranking_entries"]
pub struct ItsfRankingEntry {
    pub itsf_ranking_id: i32,
    pub place: i32,
    pub player_itsf_id: i32,
}

#[derive(Insertable)]
#[table_name = "itsf_rankings"]
pub struct NewItsfRanking {
    pub year: i32,
    pub queried_at: chrono::NaiveDateTime,
    pub count: i32,
    pub category: Option<String>,
}

#[derive(Queryable)]
pub struct ItsfRanking {
    pub id: i32,
    pub year: i32,
    pub queried_at: chrono::NaiveDateTime,
    pub count: i32,
    pub category: Option<String>,
}
