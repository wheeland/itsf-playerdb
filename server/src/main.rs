extern crate env_logger;
use std::sync::Mutex;
use actix_web::{web, App, middleware::Logger, HttpResponse, HttpServer, Responder};
use log::{debug, trace, info, warn, error};

struct AppState {
    name: Mutex<String>,
}

#[actix_web::get("/")]
async fn hello(data: web::Data<AppState>) -> impl Responder {
    let data = data.name.lock().unwrap();
    HttpResponse::Ok().body(format!("Hello world! {}", *data))
}

#[actix_web::post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[actix_web::get("/img")]
async fn img() -> impl Responder {
    HttpResponse::Ok().body("<html><body><img src=\"http://localhost:8000/k36.jpg\"/></body></html>")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    trace!("some trace log");
    debug!("some debug log");
    info!("some information log");
    warn!("some warning log");
    error!("some error log");

    let state = web::Data::new(
        AppState {
            name: Mutex::new("thy app".to_string())
        }
    );

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(state.clone())
            .service(hello)
            .service(echo)
            .service(img)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}