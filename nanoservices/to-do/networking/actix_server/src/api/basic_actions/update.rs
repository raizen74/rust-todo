use actix_web::{HttpResponse, web::Json};
use auth_kernel::api::users::get::get_user_by_unique_id;
use glue::errors::NanoServiceError;
use glue::token::HeaderToken;
use to_do_core::api::basic_actions::{get::get_all as get_all_core, update::update as update_core};
use to_do_dal::to_do_items::schema::ToDoItem;
use to_do_dal::to_do_items::transactions::{get::GetAll, update::UpdateOne};

pub async fn update<T: UpdateOne + GetAll>(
    token: HeaderToken,
    body: Json<ToDoItem>,
) -> Result<HttpResponse, NanoServiceError> {
    let user = get_user_by_unique_id(token.unique_id).await?;
    let _ = update_core::<T>(body.into_inner(), user.id).await?;
    Ok(HttpResponse::Ok().json(get_all_core::<T>(user.id).await?))
}
