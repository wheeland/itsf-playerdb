use crate::data::{itsf::PlayerCategory, Player, PlayerImage};

use super::download;
use reqwest::StatusCode;
use scraper::{ElementRef, Html, Selector};

fn get_div_with_class<'a>(root: &'a Html, class: &'static str) -> Vec<ElementRef<'a>> {
    let div_selector = Selector::parse("div").unwrap();
    root.select(&div_selector)
        .filter(|div| div.value().attr("class") == Some(class))
        .collect()
}

fn is_uppercase(word: &str) -> bool {
    word.chars().all(|c| !c.is_lowercase())
}

fn to_normalcase(word: &str) -> String {
    let mut result = String::new();

    for ch in word.chars().enumerate() {
        if ch.0 == 0 {
            result.push(ch.1);
        } else {
            for ch in ch.1.to_lowercase() {
                result.push(ch);
            }
        }
    }

    result
}

fn parse_player_info_from(itsf_id: i32, html: &Html) -> Result<Player, String> {
    let nomdujoueur = get_div_with_class(&html, "nomdujoueur");
    let nomdujoueur = nomdujoueur.first().ok_or("can't find div nomdujoueur")?;
    let name = nomdujoueur.text().next().ok_or("can't find text in nomdujoueur div")?;

    let last_name = name
        .split(" ")
        .filter(|word| !word.is_empty() && is_uppercase(word))
        .map(to_normalcase)
        .collect::<Vec<String>>()
        .join(" ");

    let first_name = name
        .split(" ")
        .filter(|word| !word.is_empty() && !is_uppercase(word))
        .collect::<Vec<&str>>()
        .join(" ");

    let span_selector = Selector::parse("span").unwrap();
    let country_code = nomdujoueur
        .select(&span_selector)
        .next()
        .ok_or("can't find country code")?;
    let country_code = country_code.text().next().ok_or("can't find country code text")?;
    if !country_code.starts_with("(") || !country_code.ends_with(")") {
        return Err(format!("invalid country code ({:?})", country_code));
    }
    let country_code = country_code[1..]
        .split(" ")
        .next()
        .ok_or(format!("invalid country code ({:?})", country_code))?;

    let contenu_typeinfojoueur = get_div_with_class(&html, "contenu_typeinfojoueur");
    if contenu_typeinfojoueur.len() < 2 {
        return Err(format!(
            "invalid number of contenu_typeinfojoueur ({})",
            contenu_typeinfojoueur.len()
        ));
    }

    let contenu_typeinfojoueur_even = get_div_with_class(&html, "contenu_typeinfojoueur even");
    if contenu_typeinfojoueur_even.len() < 1 {
        return Err(format!(
            "invalid number of contenu_typeinfojoueur ({})",
            contenu_typeinfojoueur_even.len()
        ));
    }

    let category = contenu_typeinfojoueur_even[0]
        .text()
        .next()
        .ok_or("can't find category text")?
        .trim();
    let category = PlayerCategory::try_from_str(category)?;

    let birth_year = contenu_typeinfojoueur[1].text().next().ok_or("can't find birth year")?;
    let birth_year = birth_year.parse::<i32>().unwrap_or(0);

    Ok(Player {
        itsf_id,
        first_name: first_name.into(),
        last_name: last_name.into(),
        birth_year,
        country_code: Some(country_code.into()),
        category: category.into(),
        itsf_rankings: Vec::new(),
        dtfb_id: None,
        dtfb_championship_results: Vec::new(),
        dtfb_national_rankings: Vec::new(),
        dtfb_league_teams: Vec::new(),
        comments: Vec::new(),
    })
}

async fn download_player_info_from(itsf_id: i32, url: &str) -> Result<Player, String> {
    let body = download::download(url, &[]).await?;
    let itsf = Html::parse_document(&body);
    parse_player_info_from(itsf_id, &itsf)
}

pub async fn download_player_info(itsf_id: i32) -> Result<Player, String> {
    let url = format!("https://www.tablesoccer.org/page/player&numlic={:08}", itsf_id);
    download_player_info_from(itsf_id, &url)
        .await
        .map_err(|msg| format!("Player[{}]: {}", url, msg))
}

pub async fn download_player_image(itsf_id: i32) -> Result<Option<PlayerImage>, String> {
    let url = format!("https://media.fast4foos.org/photos/players/{:08}.jpg", itsf_id);

    let response = match reqwest::get(url).await {
        Ok(response) => {
            if response.status() == StatusCode::NOT_FOUND {
                return Ok(None);
            }
            response
        }
        Err(err) => {
            if let Some(status) = err.status() {
                if status == StatusCode::NOT_FOUND {
                    return Ok(None);
                }
            }
            return Err(err.to_string());
        }
    };

    let bytes = response.bytes().await.map_err(|err| err.to_string())?;

    Ok(Some(PlayerImage {
        itsf_id,
        image_data: bytes.to_vec(),
        image_format: String::from("jpg"),
    }))
}
