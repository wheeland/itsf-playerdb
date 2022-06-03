#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate r2d2;

use std::sync::Weak;

use actix_web::{http::header, middleware::Logger, web, App, Error, HttpResponse, HttpServer};
use diesel::prelude::*;
use models::{ItsfRankingCategory, ItsfRankingClass};
use std::sync::Mutex;

type SqliteDbPool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>;

mod background;
mod json;
mod models;
mod queries;
mod schema;
mod scraping;

struct AppState {
    db_pool: SqliteDbPool,
    itsf_ranking_download: Mutex<Weak<background::BackgroundOperationProgress>>,
}

impl AppState {
    async fn execute_db_operation<F, R>(
        data: web::Data<AppState>,
        f: F,
    ) -> Result<R, actix_web::Error>
    where
        F: FnOnce(&SqliteConnection) -> R + Send + 'static,
        R: Send + 'static,
    {
        // use web::block to offload blocking Diesel code without blocking server thread
        web::block(move || {
            let conn = data.db_pool.get()?;
            let result: Result<R, r2d2::Error> = Ok(f(&conn));
            result
        })
        .await?
        .map_err(actix_web::error::ErrorInternalServerError)
    }
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

    let player = AppState::execute_db_operation(data, move |conn| {
        let player = queries::get_player(conn, itsf_lic);
        player.map(|player| {
            let itsf_rankings = queries::get_itsf_rankings(conn, itsf_lic)
                .iter()
                .map(|ranking| {
                    PlayerRankingJson {
                        year: ranking.year,
                        place: ranking.place,
                        category: ranking.category.to_str().into(),
                        class: ranking.class.to_str().into(),
                    }
                })
                .collect();

            PlayerJson {
                first_name: player.first_name,
                last_name: player.last_name,
                birth_year: player.birth_year,
                country_code: player.country_code.unwrap_or(String::new()),
                image_url: format!("/image/{}.jpg", itsf_lic),
                itsf_rankings,
            }
        })
    }).await?;

    match player {
        None => Ok(HttpResponse::BadRequest().json(json::err("no player found"))),
        Some(player) => Ok(HttpResponse::Ok().json(json::ok(player))),
    }
}

#[actix_web::get("/image/{itsf_lic}.jpg")]
async fn get_player_image(
    data: web::Data<AppState>,
    itsf_lic: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let itsf_lic = itsf_lic.into_inner();

    let player =
        AppState::execute_db_operation(data, move |conn| queries::get_player_image(conn, itsf_lic))
            .await?;
    match player {
        Some(image) => Ok(HttpResponse::Ok()
            .append_header(("Content-Type", "image/jpeg"))
            .body(image.image_data)),
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
        "open" => ItsfRankingCategory::Open,
        "women" => ItsfRankingCategory::Women,
        "senior" => ItsfRankingCategory::Senior,
        "junior" => ItsfRankingCategory::Junior,
        _ => {
            return Ok(HttpResponse::BadRequest().json(json::err(
                "Invalid category. Must be one of ['open', 'women', 'senior', 'junior'].",
            )))
        }
    };

    let class = match itsf_lic.2.to_lowercase().as_str() {
        "singles" => ItsfRankingClass::Singles,
        "doubles" => ItsfRankingClass::Doubles,
        "combined" => ItsfRankingClass::Combined,
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

    let conn = data
        .db_pool
        .get()
        .map_err(actix_web::error::ErrorInternalServerError)?;
    *itsf_ranking_download = scraping::start_itsf_rankings_download(conn, vec![year], vec![category], vec![class]);

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

    let conn = data
        .db_pool
        .get()
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let years = (2010..2022).collect();
    let categories = vec![ItsfRankingCategory::Open, ItsfRankingCategory::Women, ItsfRankingCategory::Senior, ItsfRankingCategory::Junior];
    let classes = vec![ItsfRankingClass::Singles, ItsfRankingClass::Doubles, ItsfRankingClass::Combined];
    *itsf_ranking_download = scraping::start_itsf_rankings_download(conn, years, categories, classes);

    Ok(HttpResponse::Ok().json(json::ok("Started download")))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    // Open SQLite database pool
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL missing");
    let db_manager = diesel::r2d2::ConnectionManager::<SqliteConnection>::new(database_url);
    let db_pool = r2d2::Pool::builder()
        .build(db_manager)
        .expect("Failed to create R2D2 pool.");

    let state = AppState {
        db_pool,
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
