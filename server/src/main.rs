#[macro_use]
extern crate diesel;
extern crate dotenv;

use std::sync::Mutex;
use actix_web::{web, App, middleware::Logger, HttpResponse, HttpServer, Error, Responder, HttpRequest};
use actix_web_httpauth::extractors::basic::BasicAuth;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use log::{debug, trace, info, warn, error};

type SqliteDbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

mod models;
mod schema;
mod queries;

struct AppState {
    name: Mutex<String>,
    db_pool: SqliteDbPool,
}

#[actix_web::get("/player/{itsf_lic}")]
async fn hello(
    data: web::Data<AppState>,
    itsf_lic: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let itsf_lic = itsf_lic.into_inner();
    
    // use web::block to offload blocking Diesel code without blocking server thread
    let player = web::block(move || {
        let conn = data.db_pool.get()?;
        queries::get_player(&conn, &itsf_lic)
    })
    .await?
    .map_err(actix_web::error::ErrorInternalServerError)?;

    let json = match player {
        None => "{ \"error\": \"No player found\" }".into(),
        Some(player) => {
            let json = serde_json::to_string(&player).unwrap();
            format!("{{ \"data\": {} }}", json)
        }
    };
    Ok(HttpResponse::Ok().body(json))
}

#[actix_web::get("/img")]
async fn img() -> impl Responder {
    HttpResponse::Ok().body("<html><body><img src=\"http://localhost:8000/k36.jpg\"/></body></html>")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    // Open SQLite database pool
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL missing");
    let db_manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let db_pool = r2d2::Pool::builder()
        .build(db_manager)
        .expect("Failed to create R2D2 pool.");

    let state = web::Data::new(
        AppState {
            name: Mutex::new("thy app".to_string()),
            db_pool,
        }
    );

    log::info!("Starting HTTP server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(state.clone())
            .service(hello)
            .service(img)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}