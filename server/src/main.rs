extern crate env_logger;
use std::sync::Mutex;
use actix_web::{web, App, middleware::Logger, HttpResponse, HttpServer, Responder, HttpRequest};
use actix_web_httpauth::extractors::basic::BasicAuth;
use log::{debug, trace, info, warn, error};
use maud::html;

struct AppState {
    name: Mutex<String>,
}

#[actix_web::get("/")]
async fn hello(data: web::Data<AppState>) -> impl Responder {
    let data = data.name.lock().unwrap();
    HttpResponse::Ok().body(format!("Hello world! {}", *data))
}

#[actix_web::get("/img")]
async fn img() -> impl Responder {
    HttpResponse::Ok().body("<html><body><img src=\"http://localhost:8000/k36.jpg\"/></body></html>")
}

#[actix_web::get("/prot")]
async fn prot(auth: BasicAuth, req: HttpRequest) -> impl Responder {
    let user = auth.user_id().as_ref();
    let pass = auth.password().map(|p| p.as_ref()).unwrap_or("[]");
    let logout = req.query_string().contains("logout");

    if logout || (user != pass) {
        let html = html! {
            html {
                body {
                    p {
                        "Access forbidden"
                    }
                    p {
                        a id="logout_link" href="/prot" {
                            "Refresh to login with username/password"
                        }
                    }
                }
            }
        };

        HttpResponse::Unauthorized().body(html.into_string())
    } else {
        let html = html! {
            html {
                body {
                    p {
                        "Hello user! user=" (user) " pass=" (pass) " "
                    }

                    a id="logout_link" href="?logout" {
                        "Logout"
                    }
                }
            }
        };

        HttpResponse::Ok().body(html.into_string())
    }
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
            .service(prot)
            .service(img)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}