use actix_web::{error::Error, get, web::Data, HttpResponse};

use crate::AppState;

const CACHE_KEY: &'static str = "menu";

#[get("/api/menu")]
pub async fn menu(app_state: Data<AppState>) -> Result<HttpResponse, Error> {
    let cache = &app_state.cache;
    let cache_live = cache.lock().unwrap().clone();

    let out = match cache_live.get(CACHE_KEY) {
        Some(out) => out,
        None => {
            println!("Menu not found in cache, updating data...");
            let out = amica_premium_api::get_menu_async()
                .await
                .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
            cache_live.insert(CACHE_KEY.to_string(), out.clone());
            out
        }
    };
    // Update cache
    *cache.lock().unwrap() = cache_live;

    return Ok(HttpResponse::Ok().json(out));
}
