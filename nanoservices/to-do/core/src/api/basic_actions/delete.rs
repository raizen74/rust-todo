use glue::errors::NanoServiceError;
use glue::token::HeaderToken;
use to_do_dal::to_do_items::transactions::delete::DeleteOne;

pub async fn delete<T: DeleteOne>(token: HeaderToken, id: &str) -> Result<(), NanoServiceError> {
    let _ = T::delete_one(id.to_string()).await?;
    Ok(())
}
