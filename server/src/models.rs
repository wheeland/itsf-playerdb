use super::schema::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(i32)]
pub enum PlayerCategory {
    Men,
    Women,
    JuniorMale,
    JuniorFemale,
    SeniorMale,
    SeniorFemale,
}

impl PlayerCategory {
    pub fn try_from_str(category: &str) -> Result<PlayerCategory, String> {
        match category {
            "MEN" => Ok(PlayerCategory::Men),
            "WOMEN" => Ok(PlayerCategory::Women),
            "JUNIOR MALE" => Ok(PlayerCategory::JuniorMale),
            "JUNIOR FEMALE" => Ok(PlayerCategory::JuniorFemale),
            "SENIOR MALE" => Ok(PlayerCategory::SeniorMale),
            "SENIOR FEMALE" => Ok(PlayerCategory::SeniorFemale),
            _ => Err(format!("invalid category: '{}'", category)),
        }
    }
    pub fn to_str(&self) -> &'static str {
        match *self {
            PlayerCategory::Men => "MEN",
            PlayerCategory::Women => "WOMEN",
            PlayerCategory::JuniorMale => "JUNIOR MALE",
            PlayerCategory::JuniorFemale => "JUNIOR FEMALE",
            PlayerCategory::SeniorMale => "SENIOR MALE",
            PlayerCategory::SeniorFemale => "SENIOR FEMALE",
        }
    }
}

#[derive(Debug, Clone, Insertable, Queryable, serde::Serialize, serde::Deserialize)]
pub struct Player {
    pub itsf_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub dtfb_license: Option<String>,
    pub birth_year: i32,
    pub country_code: Option<String>,
    pub category: i32,
}

#[derive(Insertable, Queryable)]
pub struct PlayerImage {
    pub itsf_id: i32,
    pub image_data: Vec<u8>,
    pub image_format: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(i32)]
pub enum ItsfRankingCategory {
    Open,
    Women,
    Junior,
    Senior,
}

impl ItsfRankingCategory {
    pub fn to_str(&self) -> &'static str {
        match *self {
            ItsfRankingCategory::Open => "open",
            ItsfRankingCategory::Women => "women",
            ItsfRankingCategory::Junior => "junior",
            ItsfRankingCategory::Senior => "senior",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(i32)]
pub enum ItsfRankingClass {
    Singles,
    Doubles,
    Combined,
}

impl ItsfRankingClass {
    pub fn to_str(&self) -> &'static str {
        match *self {
            ItsfRankingClass::Singles => "singles",
            ItsfRankingClass::Doubles => "doubles",
            ItsfRankingClass::Combined => "combined",
        }
    }
}

#[derive(Insertable)]
#[table_name = "itsf_rankings"]
pub struct NewItsfRanking {
    pub year: i32,
    pub queried_at: chrono::NaiveDateTime,
    pub count: i32,
    pub category: i32,
    pub class: i32,
}

#[derive(Debug, Clone, Queryable, Insertable)]
#[table_name = "itsf_ranking_entries"]
pub struct ItsfRankingEntry {
    pub itsf_ranking_id: i32,
    pub place: i32,
    pub player_itsf_id: i32,
}

#[derive(Queryable)]
pub struct ItsfRanking {
    pub id: i32,
    pub year: i32,
    pub queried_at: chrono::NaiveDateTime,
    pub count: i32,
    pub category: Option<String>,
}
