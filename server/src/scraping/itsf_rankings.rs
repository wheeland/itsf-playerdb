use super::download;
use crate::models::{ItsfRankingCategory, ItsfRankingClass};
use scraper::{ElementRef, Selector};

fn get_player_from_div(div: &ElementRef) -> Result<(i32, i32), &'static str> {
    let id = div.value().attr("id").ok_or("no id attr")?;
    let onclick = div.value().attr("onclick").ok_or("no onclick attr")?;

    let place = if id.starts_with("place") {
        id[5..]
            .parse::<i32>()
            .map_err(|_| "can't parse place attr")?
    } else {
        Err("id attr has no place")?
    };

    let license = if onclick.contains("&numlic=") {
        let mut parts = onclick.split("&numlic=");
        parts.next().ok_or("onclick doesn't contain player link")?;
        let license = parts.next().ok_or("onclick doesn't contain player link")?;
        license
            .split("&")
            .next()
            .ok_or("doesn't contain player link")?
            .parse::<i32>()
            .map_err(|_| "can't parse player license")?
    } else {
        Err("onclick doesn't contain player link")?
    };

    Ok((place, license))
}

pub async fn download(
    year: i32,
    category: ItsfRankingCategory,
    class: ItsfRankingClass,
) -> Result<Vec<(i32, i32)>, String> {
    let category = match category {
        ItsfRankingCategory::Open => "o",
        ItsfRankingCategory::Women => "w",
        ItsfRankingCategory::Junior => "j",
        ItsfRankingCategory::Senior => "s",
    };
    let class = match class {
        ItsfRankingClass::Singles => "s",
        ItsfRankingClass::Doubles => "d",
        ItsfRankingClass::Combined => "c",
    };
    let url = format!("https://www.tablesoccer.org/page/rankings?category={}{}&system=1&Ranking+Rules=Select+Category&tour={}&vues=1000", category, class, year);
    let itsf = download::download(&url).await?;

    let mut ret = Vec::new();

    let div_selector = Selector::parse("div").unwrap();
    for div in itsf.select(&div_selector) {
        if let Ok(placement) = get_player_from_div(&div) {
            ret.push(placement);
        }
    }

    Ok(ret)
}
