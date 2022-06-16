use scraper::Selector;

use crate::data::dtfb::*;

use super::download;

pub async fn collect_dtfb_ids_from_rankings(ranking_id: i32) -> Result<Vec<i32>, String> {
    let url = format!(
        "https://dtfb.de/wettbewerbe/turnierserie/rangliste?task=rangliste&id={}",
        ranking_id
    );
    let itsf = download::download_html(&url).await?;

    let mut ret = Vec::new();

    for a in itsf.select(&Selector::parse("a").unwrap()) {
        if let Some(href) = a.value().attr("href") {
            let parts: Vec<&str> = href.split("?task=spieler_details&id=").collect();
            if parts.len() == 2 {
                match parts[1].parse::<i32>() {
                    Ok(id) => ret.push(id),
                    Err(_) => log::error!("failed to parse DTFB player id: {}", href),
                }
            }
        }
    }

    Ok(ret)
}

pub struct DtfbPlayerInfo {
    pub dtfb_id: i32,
    pub itsf_id: i32,
    pub championship_results: Vec<NationalChampionshipResult>,
    pub national_rankings: Vec<NationalRanking>,
    pub teams: Vec<(i32, String)>,
}

fn value<'a>(json: &'a serde_json::Value, name: &str) -> Result<&'a serde_json::Value, String> {
    json.get(name).ok_or(format!("Can't find field {}", name))
}

fn int(json: &serde_json::Value, name: &str) -> Result<i32, String> {
    let value = value(json, name)?;
    if let Some(int) = value.as_i64() {
        Ok(int as i32)
    } else if let Some(st) = value.as_str() {
        st.parse::<i32>()
            .map_err(|err| format!("not an int: {}: '{}'", name, st))
    } else {
        Err(format!("not an int: {}", name))
    }
}

fn string<'a>(json: &'a serde_json::Value, name: &str) -> Result<&'a str, String> {
    value(json, name)?
        .as_str()
        .ok_or(format!("not a string: {}", name))
}

fn array<'a>(
    json: &'a serde_json::Value,
    name: &str,
) -> Result<&'a Vec<serde_json::Value>, String> {
    value(json, name)?
        .as_array()
        .ok_or(format!("Not an array: {}", name))
}

impl DtfbPlayerInfo {
    pub async fn download(dtfb_id: i32) -> Result<Self, String> {
        let url = format!(
            "https://dtfb.de/component/sportsmanager?task=spieler_details&id={}&format=json",
            dtfb_id
        );
        let json = download::download(&url).await?;
        let json: serde_json::Value = serde_json::from_str(&json).map_err(|err| err.to_string())?;

        let data = value(&json, "data")?;
        let spieler = value(data, "spieler")?;
        let spieler_id = int(spieler, "spieler_id")?;
        let lizenznr = int(spieler, "lizenznr")?;
        let teams = array(data, "teams")?;
        let turnier_platzierungen = array(data, "turnier_platzierungen")?;
        let ranglisten_platzierungen = array(data, "ranglisten_platzierungen")?;

        if spieler_id != dtfb_id {
            return Err(format!(
                "DTFB player id doesn't match: {} vs {}",
                dtfb_id, spieler_id
            ));
        }

        let mut player_teams = Vec::new();
        for team in teams {
            let saisonbezeichnung = int(team, "saisonbezeichnung")?;
            let teamname = string(team, "teamname")?;
            let bezeichnung = string(team, "bezeichnung")?;
            if bezeichnung.contains("undesliga") {
                player_teams.push((saisonbezeichnung, String::from(teamname)));
            }
        }

        let mut championship_results = Vec::new();
        for placement in turnier_platzierungen {
            let saisonbezeichnung = int(placement, "saisonbezeichnung")?;
            let turnierbezeichnung = string(placement, "turnierbezeichnung")?;
            let disziplin = string(placement, "disziplin")?;
            let platz = int(placement, "platz")?;
            if turnierbezeichnung == "Deutsche Meisterschaft" {
                let class = if disziplin.contains("Einzel") {
                    Some(ChampionshipClass::Singles)
                } else if disziplin.contains("Doppel") {
                    Some(ChampionshipClass::Doubles)
                } else {
                    None
                };
                let category = if disziplin.contains("Herren") {
                    Some(ChampionshipCategory::Men)
                } else if disziplin.contains("Damen") {
                    Some(ChampionshipCategory::Women)
                } else if disziplin.contains("Junior") {
                    Some(ChampionshipCategory::Junior)
                } else if disziplin.contains("Senior") {
                    Some(ChampionshipCategory::Senior)
                } else {
                    None
                };

                if let Some((class, category)) = class.zip(category) {
                    championship_results.push(NationalChampionshipResult {
                        place: platz as _,
                        year: saisonbezeichnung as _,
                        class,
                        category,
                    })
                }
            }
        }

        let mut national_rankings = Vec::new();
        for ranking in ranglisten_platzierungen {
            let saisonbezeichnung = int(ranking, "saisonbezeichnung")?;
            let platz = int(ranking, "platz")?;
            let bezeichnung = string(ranking, "bezeichnung")?;
            let category = match bezeichnung {
                "Herren" => Some(ChampionshipCategory::Men),
                "Damen" => Some(ChampionshipCategory::Women),
                "Junioren" => Some(ChampionshipCategory::Junior),
                "Senioren" => Some(ChampionshipCategory::Senior),
                _ => None,
            };
            if let Some(category) = category {
                national_rankings.push(NationalRanking {
                    year: saisonbezeichnung as _,
                    place: platz as _,
                    category,
                });
            }
        }

        Ok(DtfbPlayerInfo {
            dtfb_id,
            itsf_id: lizenznr,
            championship_results,
            national_rankings,
            teams: player_teams,
        })
    }
}
