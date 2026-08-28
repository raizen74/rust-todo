use actix_web::HttpResponse;
// use auth_kernel::api::users::get::get_user_by_unique_id;
use auth_kernel::user_session::transactions::get::GetUserSession;
use glue::errors::NanoServiceError;
use glue::token::HeaderToken;
use to_do_core::api::basic_actions::get::get_all as get_all_core;
use to_do_dal::to_do_items::transactions::get::GetAll;

pub async fn get_all<T, X>(token: HeaderToken) -> Result<HttpResponse, NanoServiceError>
where
    T: GetAll,
    X: GetUserSession,
{
    let session = X::get_user_session(token.unique_id).await?;  // retrieve the session from the cache
    Ok(HttpResponse::Ok().json(get_all_core::<T>(session.user_id).await?))
}
