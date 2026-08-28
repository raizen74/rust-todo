use actix_web::{HttpRequest, HttpResponse};
// use auth_kernel::api::users::get::get_user_by_unique_id;
use auth_kernel::user_session::transactions::get::GetUserSession;
use glue::errors::{NanoServiceError, NanoServiceErrorStatus};
use glue::token::HeaderToken;
use to_do_core::api::basic_actions::{delete::delete as delete_core, get::get_all as get_all_core};
use to_do_dal::to_do_items::transactions::{delete::DeleteOne, get::GetAll};

pub async fn delete_by_name<T, X>(
    token: HeaderToken,
    req: HttpRequest,
) -> Result<HttpResponse, NanoServiceError>
where
    T: DeleteOne + GetAll,
    X: GetUserSession,
{
    let session = X::get_user_session(token.unique_id).await?;  // retrieve the session from the cache
    match req.match_info().get("name") {
        Some(name) => {
            delete_core::<T>(name, session.user_id).await?;
        }
        None => {
            return Err(NanoServiceError::new(
                "Name not provided".to_string(),
                NanoServiceErrorStatus::BadRequest,
            ));
        }
    };
    Ok(HttpResponse::Ok().json(get_all_core::<T>(session.user_id).await?))
}
