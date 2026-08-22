use actix_web::HttpResponse;
use glue::errors::NanoServiceError;
use to_do_core::api::basic_actions::get::get_all as get_all_core;

pub async fn get_all() -> Result<HttpResponse, NanoServiceError> {
    Ok(HttpResponse::Ok().json(get_all_core().await?))
}
