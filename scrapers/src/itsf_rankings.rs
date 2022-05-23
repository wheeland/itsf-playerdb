use scraper::{Selector, ElementRef};
use crate::download::download;

fn get_player_from_div(div: &ElementRef) -> Result<(u32, String),&'static str> {
    let id = div.value().attr("id").ok_or("no id attr")?;
    let onclick = div.value().attr("onclick").ok_or("no onclick attr")?;

    let place = if id.starts_with("place") {
        id[5..].parse::<u32>().map_err(|_| "can't parse place attr")?
    } else {
        Err("id attr has no place")?
    };

    let license = if onclick.contains("&numlic=") {
        let mut parts = onclick.split("&numlic=");
        parts.next().ok_or("onclick doesn't contain player link")?;
        let license = parts.next().ok_or("onclick doesn't contain player link")?;
        license.split("&").next().ok_or("doesn't contain player link")?
    } else {
        Err("onclick doesn't contain player link")?
    };

    Ok((place, license.to_string()))
}

#[derive(Debug, Clone, Copy)]
pub enum ItsfRanking {
    Singles,
    Doubles,
    Combined,
}

pub struct Placement {
    pub place: u32,
    pub lic: String,
}

pub async fn get_itsf_rankings(year: u32, ranking: ItsfRanking) -> Result<Vec<Placement>, String> {
    let category = match ranking {
        ItsfRanking::Combined => "oc",
        ItsfRanking::Singles => "os",
        ItsfRanking::Doubles => "od",
    };
    let url = format!("https://www.tablesoccer.org/page/rankings?category={}&system=1&Ranking+Rules=Select+Category&tour={}&vues=1000", category, year);
    let itsf = download(&url).await?;

    let mut ret = Vec::new();

    let div_selector = Selector::parse("div").unwrap();
    for div in itsf.select(&div_selector) {
        if let Ok((place, lic)) = get_player_from_div(&div) {
            ret.push(Placement {
                place,
                lic,
            });
        }
    }

    Ok(ret)
}
