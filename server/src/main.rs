#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate r2d2;

use actix_web::{middleware::Logger, web, App, Error, HttpResponse, HttpServer, Responder};
//use actix_web_httpauth::extractors::basic::BasicAuth;
use diesel::prelude::*;

type SqliteDbPool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>;

mod models;
mod queries;
mod schema;

struct AppState {
    db_pool: SqliteDbPool,
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
async fn hello(data: web::Data<AppState>, itsf_lic: web::Path<i32>) -> Result<HttpResponse, Error> {
    let itsf_lic = itsf_lic.into_inner();

    let player =
        AppState::execute_db_operation(data, move |conn| queries::get_player(conn, itsf_lic))
            .await?;

    let json = match player {
        None => "{ \"error\": \"No player found\" }".into(),
        Some(player) => {
            let json = serde_json::to_string(&player).unwrap();
            format!("{{ \"data\": {} }}", json)
        }
    };
    Ok(HttpResponse::Ok().body(json))
}

#[actix_web::get("/addplayer/{itsf_lic}/{first_name}/{last_name}")]
async fn add_player(
    data: web::Data<AppState>,
    itsf_lic: web::Path<(i32, String, String)>,
) -> Result<HttpResponse, Error> {
    let (itsf_lic, first_name, last_name) = itsf_lic.into_inner();

    let ok = AppState::execute_db_operation(data, move |conn| {
        queries::add_player(
            &conn,
            models::Player {
                itsf_id: itsf_lic,
                first_name: first_name,
                last_name: last_name,
                dtfb_license: None,
                birth_year: 1234,
                country_code: Some("GER".into()),
                category: models::PlayerCategory::Men.into(),
            },
        )
    })
    .await?;

    let json = if ok {
        "{ \"data\": true }"
    } else {
        "{ \"error\": \"player already exists\" }".into()
    };
    Ok(HttpResponse::Ok().body(json))
}

#[actix_web::get("/img")]
async fn img() -> impl Responder {
    HttpResponse::Ok()
        .body("<html><body><img src=\"http://localhost:8000/k36.jpg\"/></body></html>")
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

    let state = web::Data::new(AppState { db_pool });

    let ok = AppState::execute_db_operation(state.clone(), move |conn| {
        let d = chrono::NaiveDate::from_ymd(2015, 6, 3);
        let t = chrono::NaiveTime::from_hms_milli(12, 34, 56, 789);
        let dt = chrono::NaiveDateTime::new(d, t);

        queries::add_itsf_rankings(
            &conn,
            2012,
            dt,
            models::ItsfRankingCategory::Men,
            models::ItsfRankingClass::Doubles,
            &[(1, 2), (3, 4)],
        );
    })
    .await;

    log::info!("Starting HTTP server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(state.clone())
            .service(hello)
            .service(add_player)
            .service(img)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
