use actix_web::{HttpResponse, web::Json};
use glue::{errors::NanoServiceError, token::HeaderToken};
use to_do_core::api::basic_actions::{get::get_all as get_all_core, update::update as update_core};
use to_do_core::structs::ToDoItem;

pub async fn update(
    token: HeaderToken,
    body: Json<ToDoItem>,
) -> Result<HttpResponse, NanoServiceError> {
    println!("Token: {}", token.message);
    let _ = update_core(body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(get_all_core().await?))
}
