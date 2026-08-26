use glue::errors::NanoServiceError;
use to_do_dal::to_do_items::schema::{NewToDoItem, ToDoItem};
use to_do_dal::to_do_items::transactions::create::SaveOne;

pub async fn create<T: SaveOne>(item: NewToDoItem) -> Result<ToDoItem, NanoServiceError> {
    // Call the SaveOne trait's save_one method, implemented in nanoservices/dal/src/to_do_items/transactions/create.rs
    // in the SqlxPostGresDescriptor and JsonFileDescriptor structs, to save the new ToDoItem to the database or JSON file.
    let created_item = T::save_one(item).await?;
    Ok(created_item)
}
