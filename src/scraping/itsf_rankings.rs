use super::download;
use crate::data::itsf::*;
use scraper::{ElementRef, Selector};

fn get_player_from_div(div: &ElementRef) -> Result<(i32, i32), &'static str> {
    let id = div.value().attr("id").ok_or("no id attr")?;
    let onclick = div.value().attr("onclick").ok_or("no onclick attr")?;

    let place = if let Some(striped_place) = id.strip_prefix("place") {
        striped_place.parse::<i32>().map_err(|_| "can't parse place attr")?
    } else {
        Err("id attr has no place")?
    };

    let license = if onclick.contains("&numlic=") {
        let mut parts = onclick.split("&numlic=");
        parts.next().ok_or("onclick doesn't contain player link")?;
        let license = parts.next().ok_or("onclick doesn't contain player link")?;
        license
            .split('&')
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
    category: RankingCategory,
    class: RankingClass,
    count: usize,
) -> Result<Vec<(i32, i32)>, String> {
    let category = match category {
        RankingCategory::Open => "o",
        RankingCategory::Women => "w",
        RankingCategory::Junior => "j",
        RankingCategory::Senior => "s",
    };
    let class = match class {
        RankingClass::Singles => "s",
        RankingClass::Doubles => "d",
        RankingClass::Combined => "c",
    };
    let url = format!("https://www.tablesoccer.org/page/rankings?category={}{}&system=1&Ranking+Rules=Select+Category&tour={}&vues={}", category, class, year, count);
    let itsf = download::download_html(&url).await?;

    let mut ret = Vec::new();

    let div_selector = Selector::parse("div").unwrap();
    for div in itsf.select(&div_selector) {
        if let Ok(placement) = get_player_from_div(&div) {
            ret.push(placement);
        }
    }

    Ok(ret)
}
