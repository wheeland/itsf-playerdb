#[macro_use]
extern crate diesel;

use crate::data::{dtfb, itsf};
use actix_web::{middleware::Logger, web, App, Error, HttpResponse, HttpServer};
use std::sync::{Mutex, Weak};

mod background;
mod data;
mod json;
mod schema;
mod scraping;

struct AppState {
    data: data::DatabaseRef,
    download: Mutex<Weak<background::BackgroundOperationProgress>>,
}

#[actix_web::get("/player/{itsf_lic}")]
async fn get_player(data: web::Data<AppState>, itsf_lic: web::Path<i32>) -> Result<HttpResponse, Error> {
    let itsf_lic = itsf_lic.into_inner();

    #[derive(serde::Serialize)]
    struct PlayerJson {
        pub first_name: String,
        pub last_name: String,
        pub birth_year: i32,
        pub country_code: String,
        pub image_url: String,
        pub itsf_rankings: Vec<itsf::Ranking>,
        pub dtfb_rankings: Vec<dtfb::NationalRanking>,
        pub dm_placements: Vec<dtfb::NationalChampionshipResult>,
        pub dtfl_teams: Vec<(i32, String)>,
    }

    match data.data.get_player(itsf_lic) {
        Some(player) => {
            let player = PlayerJson {
                first_name: player.first_name,
                last_name: player.last_name,
                birth_year: player.birth_year,
                country_code: player.country_code.unwrap_or(String::new()),
                image_url: format!("/image/{}.jpg", itsf_lic),
                itsf_rankings: player.itsf_rankings,
                dtfb_rankings: player.dtfb_national_rankings,
                dm_placements: player.dtfb_championship_results,
                dtfl_teams: player.dtfb_league_teams,
            };

            Ok(HttpResponse::Ok().json(json::ok(player)))
        }
        None => Ok(HttpResponse::NotFound().json(json::err("No such player"))),
    }
}

#[actix_web::get("/image/{itsf_lic}.jpg")]
async fn get_player_image(data: web::Data<AppState>, itsf_lic: web::Path<i32>) -> Result<HttpResponse, Error> {
    let itsf_lic = itsf_lic.into_inner();

    match data.data.get_player_image(itsf_lic) {
        Some(player_image) => Ok(HttpResponse::Ok()
            .append_header(("Content-Type", "image/jpeg"))
            .body(player_image.image_data)),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

fn download_itsf(data: web::Data<AppState>, years: Vec<i32>) -> Result<HttpResponse, Error> {
    let mut download = data
        .download
        .lock()
        .map_err(|_| actix_web::error::ErrorInternalServerError("internal lock"))?;

    if let Some(_) = download.upgrade() {
        return Ok(HttpResponse::BadRequest().json(json::err("Ranking query still in progress")));
    }

    let categories = vec![
        itsf::RankingCategory::Open,
        itsf::RankingCategory::Women,
        itsf::RankingCategory::Senior,
        itsf::RankingCategory::Junior,
    ];
    let classes = vec![
        itsf::RankingClass::Singles,
        itsf::RankingClass::Doubles,
        itsf::RankingClass::Combined,
    ];
    *download = scraping::start_itsf_rankings_download(data.data.clone(), years, categories, classes);

    Ok(HttpResponse::Ok().json(json::ok("Started download")))
}

#[actix_web::get("/download_itsf/{year}")]
async fn download_itsf_single(data: web::Data<AppState>, year: web::Path<i32>) -> Result<HttpResponse, Error> {
    let year = year.into_inner();
    let year = if year > 2006 {
        year
    } else {
        return Ok(HttpResponse::BadRequest().json(json::err("Invalid year")));
    };

    download_itsf(data, vec![year])
}

#[actix_web::get("/download_itsf_all")]
async fn download_all_itsf(data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let years = (2010..2022).collect();
    download_itsf(data, years)
}

fn download_dtfb(data: web::Data<AppState>, seasons: Vec<i32>) -> Result<HttpResponse, Error> {
    let mut download = data
        .download
        .lock()
        .map_err(|_| actix_web::error::ErrorInternalServerError("internal lock"))?;

    if let Some(_) = download.upgrade() {
        return Ok(HttpResponse::BadRequest().json(json::err("Ranking query still in progress")));
    }

    *download = scraping::start_dtfb_rankings_download(data.data.clone(), seasons);

    Ok(HttpResponse::Ok().json(json::ok("Started download")))
}

#[actix_web::get("/download_dtfb/{season}")]
async fn download_dtfb_single(data: web::Data<AppState>, season: web::Path<i32>) -> Result<HttpResponse, Error> {
    let season = season.into_inner();
    let season = if season > 2008 {
        season
    } else {
        return Ok(HttpResponse::BadRequest().json(json::err("Invalid season")));
    };
    download_dtfb(data, vec![season])
}

#[actix_web::get("/download_dtfb_all")]
async fn download_dtfb_all(data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let seasons = (2010..2022).collect();
    download_dtfb(data, seasons)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let database_path = std::env::var("DATABASE_URL").expect("DATABASE_URL missing");
    let state = AppState {
        data: data::DatabaseRef::load(&database_path),
        download: Mutex::new(Weak::new()),
    };
    let state = web::Data::new(state);

    log::info!("Starting HTTP server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(state.clone())
            .service(get_player)
            .service(get_player_image)
            .service(download_itsf_single)
            .service(download_all_itsf)
            .service(download_dtfb_single)
            .service(download_dtfb_all)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
