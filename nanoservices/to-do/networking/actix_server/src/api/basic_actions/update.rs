use actix_web::{HttpResponse, web::Json};
// use auth_kernel::api::users::get::get_user_by_unique_id;
use auth_kernel::user_session::transactions::get::GetUserSession;
use glue::errors::NanoServiceError;
use glue::token::HeaderToken;
use to_do_core::api::basic_actions::{get::get_all as get_all_core, update::update as update_core};
use to_do_dal::to_do_items::schema::ToDoItem;
use to_do_dal::to_do_items::transactions::{get::GetAll, update::UpdateOne};

pub async fn update<T, X>(
    token: HeaderToken,
    body: Json<ToDoItem>,
) -> Result<HttpResponse, NanoServiceError>
where
    T: UpdateOne + GetAll,
    X: GetUserSession,
{
    let session = X::get_user_session(token.unique_id).await?;  // retrieve the session from the cache
    let _ = update_core::<T>(body.into_inner(), session.user_id).await?;
    Ok(HttpResponse::Ok().json(get_all_core::<T>(session.user_id).await?))
}
