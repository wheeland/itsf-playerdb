#[macro_use]
extern crate diesel;

use crate::data::itsf;
use actix_web::{middleware::Logger, web, App, Error, HttpResponse, HttpServer};
use std::sync::{Mutex, Weak};

mod background;
mod data;
mod json;
mod schema;
mod scraping;

struct AppState {
    data: data::DatabaseRef,
    itsf_ranking_download: Mutex<Weak<background::BackgroundOperationProgress>>,
}

#[actix_web::get("/player/{itsf_lic}")]
async fn get_player(
    data: web::Data<AppState>,
    itsf_lic: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let itsf_lic = itsf_lic.into_inner();

    #[derive(serde::Serialize)]
    struct PlayerRankingJson {
        year: i32,
        place: i32,
        category: String,
        class: String,
    }

    #[derive(serde::Serialize)]
    struct PlayerJson {
        pub first_name: String,
        pub last_name: String,
        pub birth_year: i32,
        pub country_code: String,
        pub image_url: String,
        pub itsf_rankings: Vec<PlayerRankingJson>,
    }

    match data.data.get_player(itsf_lic) {
        Some(player) => {
            let itsf_rankings = player
                .itsf_rankings
                .iter()
                .map(|ranking| PlayerRankingJson {
                    year: ranking.year as _,
                    place: ranking.place as _,
                    category: ranking.category.to_str().into(),
                    class: ranking.class.to_str().into(),
                })
                .collect();

            let player = PlayerJson {
                first_name: player.first_name,
                last_name: player.last_name,
                birth_year: player.birth_year,
                country_code: player.country_code.unwrap_or(String::new()),
                image_url: format!("/image/{}.jpg", itsf_lic),
                itsf_rankings,
            };

            Ok(HttpResponse::Ok().json(json::ok(player)))
        }
        None => Ok(HttpResponse::NotFound().json(json::err("No such player"))),
    }
}

#[actix_web::get("/image/{itsf_lic}.jpg")]
async fn get_player_image(
    data: web::Data<AppState>,
    itsf_lic: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let itsf_lic = itsf_lic.into_inner();

    match data.data.get_player_image(itsf_lic) {
        Some(player_image) => Ok(HttpResponse::Ok()
            .append_header(("Content-Type", "image/jpeg"))
            .body(player_image.image_data)),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

#[actix_web::get("/download/{year}/{category}/{class}")]
async fn download_itsf(
    data: web::Data<AppState>,
    itsf_lic: web::Path<(i32, String, String)>,
) -> Result<HttpResponse, Error> {
    let year = if itsf_lic.0 > 2006 {
        itsf_lic.0
    } else {
        return Ok(HttpResponse::BadRequest().json(json::err("Invalid year")));
    };

    let category = match itsf_lic.1.to_lowercase().as_str() {
        "open" => itsf::RankingCategory::Open,
        "women" => itsf::RankingCategory::Women,
        "senior" => itsf::RankingCategory::Senior,
        "junior" => itsf::RankingCategory::Junior,
        _ => {
            return Ok(HttpResponse::BadRequest().json(json::err(
                "Invalid category. Must be one of ['open', 'women', 'senior', 'junior'].",
            )))
        }
    };

    let class = match itsf_lic.2.to_lowercase().as_str() {
        "singles" => itsf::RankingClass::Singles,
        "doubles" => itsf::RankingClass::Doubles,
        "combined" => itsf::RankingClass::Combined,
        _ => {
            return Ok(HttpResponse::BadRequest().json(json::err(
                "Invalid class. Must be one of ['singles', 'doubles', 'combined'].",
            )))
        }
    };

    let mut itsf_ranking_download = data
        .itsf_ranking_download
        .lock()
        .map_err(|_| actix_web::error::ErrorInternalServerError("internal lock"))?;

    if let Some(_) = itsf_ranking_download.upgrade() {
        return Ok(HttpResponse::BadRequest().json(json::err("Ranking query still in progress")));
    }

    *itsf_ranking_download = scraping::start_itsf_rankings_download(
        data.data.clone(),
        vec![year],
        vec![category],
        vec![class],
    );

    Ok(HttpResponse::Ok().json(json::ok("Started download")))
}

#[actix_web::get("/download_all")]
async fn download_all_itsf(data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let mut itsf_ranking_download = data
        .itsf_ranking_download
        .lock()
        .map_err(|_| actix_web::error::ErrorInternalServerError("internal lock"))?;

    if let Some(_) = itsf_ranking_download.upgrade() {
        return Ok(HttpResponse::BadRequest().json(json::err("Ranking query still in progress")));
    }

    let years = (2010..2022).collect();
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
    *itsf_ranking_download =
        scraping::start_itsf_rankings_download(data.data.clone(), years, categories, classes);

    Ok(HttpResponse::Ok().json(json::ok("Started download")))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let database_path = std::env::var("DATABASE_URL").expect("DATABASE_URL missing");
    let state = AppState {
        data: data::DatabaseRef::load(&database_path),
        itsf_ranking_download: Mutex::new(Weak::new()),
    };
    let state = web::Data::new(state);

    log::info!("Starting HTTP server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(state.clone())
            .service(get_player)
            .service(get_player_image)
            .service(download_itsf)
            .service(download_all_itsf)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
