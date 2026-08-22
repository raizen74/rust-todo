use actix_web::{HttpResponse, web::Json};
use glue::errors::NanoServiceError;
use to_do_core::api::basic_actions::{create::create as create_core, get::get_all as get_all_core};
use to_do_core::structs::ToDoItem;

// If a ToDoItem struct cannot be constructed from the JSON body of the HTTP request, 
// then a bad request response is returned to the client with the serialization error message
pub async fn create(body: Json<ToDoItem>) -> Result<HttpResponse, NanoServiceError> {
    let _ = create_core(body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(get_all_core().await?)) // load and return all the data
}
