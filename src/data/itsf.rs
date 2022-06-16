#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(i8)]
pub enum PlayerCategory {
    Men,
    Women,
    JuniorMale,
    JuniorFemale,
    SeniorMale,
    SeniorFemale,
}

impl PlayerCategory {
    pub fn try_from_str(category: &str) -> Result<Self, String> {
        match category {
            "MEN" => Ok(Self::Men),
            "WOMEN" => Ok(Self::Women),
            "JUNIOR MALE" => Ok(Self::JuniorMale),
            "JUNIOR FEMALE" => Ok(Self::JuniorFemale),
            "SENIOR MALE" => Ok(Self::SeniorMale),
            "SENIOR FEMALE" => Ok(Self::SeniorFemale),
            _ => Err(format!("invalid category: '{}'", category)),
        }
    }
    pub fn to_str(&self) -> &'static str {
        match *self {
            Self::Men => "MEN",
            Self::Women => "WOMEN",
            Self::JuniorMale => "JUNIOR MALE",
            Self::JuniorFemale => "JUNIOR FEMALE",
            Self::SeniorMale => "SENIOR MALE",
            Self::SeniorFemale => "SENIOR FEMALE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[repr(i8)]
pub enum RankingCategory {
    Open,
    Women,
    Junior,
    Senior,
}

impl RankingCategory {
    pub fn to_str(&self) -> &'static str {
        match *self {
            Self::Open => "open",
            Self::Women => "women",
            Self::Junior => "junior",
            Self::Senior => "senior",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(i8)]
pub enum RankingClass {
    Singles,
    Doubles,
    Combined,
}

impl RankingClass {
    pub fn to_str(&self) -> &'static str {
        match *self {
            Self::Singles => "singles",
            Self::Doubles => "doubles",
            Self::Combined => "combined",
        }
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Ranking {
    pub year: i32,
    pub place: i32,
    pub category: RankingCategory,
    pub class: RankingClass,
}

impl Ranking {
    pub fn matches(&self, other_ranking: &Self) -> bool {
        self.year == other_ranking.year
            && self.category == other_ranking.category
            && self.class == other_ranking.class
    }
}
