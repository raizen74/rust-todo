use glue::errors::NanoServiceError;
use glue::token::HeaderToken;
use to_do_dal::to_do_items::schema::ToDoItem;
use to_do_dal::to_do_items::transactions::update::UpdateOne;

pub async fn update<T: UpdateOne>(token: HeaderToken, item: ToDoItem) -> Result<(), NanoServiceError> {
    let _ = T::update_one(item).await?;
    Ok(())
}
