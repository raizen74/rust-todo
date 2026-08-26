use actix_web::{web::Json, HttpResponse};
use auth_core::api::users::create::{create as create_core, CreateUser};
use auth_dal::users::transactions::create::SaveOne;
use glue::errors::NanoServiceError;

pub async fn create<T: SaveOne>(body: Json<CreateUser>) -> Result<HttpResponse, NanoServiceError> {
    let _ = create_core::<T>(body.into_inner()).await?;
    Ok(HttpResponse::Created().finish())
}
