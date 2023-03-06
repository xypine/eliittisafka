use std::sync::Mutex;
use std::time::Duration;

use actix_cors::Cors;
use actix_web::{get, http::Error, middleware, web::Data, App, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};

pub const MAX_CACHE_DURATION: Duration = Duration::from_secs(1800); // 30 minutes
use moka::sync::Cache as GenericCache;
pub type Cache = GenericCache<String, amica_premium_api::Menu>;

mod handlers;

pub struct AppState {
    pub cache: Mutex<Cache>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Stats {}
#[derive(Serialize, Debug)]
pub struct Greet {
    pub endpoints: [&'static str; 2],
    pub stats: Stats,
    pub app_version: &'static str,
    pub app_license: &'static str,
}
#[get("/")]
async fn greet(_app_state: Data<AppState>) -> Result<HttpResponse, Error> {
    let out = Greet {
        stats: Stats {},
        endpoints: ["/", "/api/menu"],
        app_version: env!("CARGO_PKG_VERSION"),
        app_license: env!("CARGO_PKG_LICENSE"),
    };
    return Ok::<HttpResponse, Error>(HttpResponse::Ok().json(out));
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    let cache = Cache::builder()
        .max_capacity(1)
        .time_to_live(MAX_CACHE_DURATION)
        .build();
    let app_state = Data::new(AppState {
        cache: Mutex::new(cache),
    });

    let http_bind = std::env::var("HTTP_BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    println!("Starting to listen on http://{}", http_bind);

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            // Enable CORS
            .wrap(Cors::permissive())
            // Enable logger
            .wrap(middleware::Logger::default())
            .service(greet)
            .service(handlers::menu)
    })
    .bind(http_bind)?
    .run()
    .await
}
