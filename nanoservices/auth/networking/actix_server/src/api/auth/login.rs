use crate::extract_auth::extract_credentials;
use actix_web::{HttpRequest, HttpResponse};
use auth_core::api::auth::login::login as core_login;
use auth_dal::users::transactions::get::GetByEmail;
use auth_kernel::user_session::transactions::login::LoginUserSession;
use glue::errors::{NanoServiceError, NanoServiceErrorStatus};

pub async fn login<T, X>(req: HttpRequest) -> Result<HttpResponse, NanoServiceError>
where
    T: GetByEmail,
    X: LoginUserSession,
{
    println!("login path executed");
    let credentials = extract_credentials(req).await?;
    let token = core_login::<T>(credentials.email.clone(), credentials.password).await?;
    let user = T::get_by_email(credentials.email).await?;
    let url = std::env::var("CACHE_API_URL")
        .map_err(|e| NanoServiceError::new(e.to_string(), NanoServiceErrorStatus::Unknown))?;
    println!("Redis URL: {}", url);
    // After hitting the login endpoint we store the user session in the cache
    let _ = X::login_user_session(&url, &user.unique_id, 20, user.id).await?;
    Ok(HttpResponse::Ok().json(token))
}
