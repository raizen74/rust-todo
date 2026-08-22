use crate::structs::{AllToDoItems, ToDoItem};
use glue::errors::{NanoServiceError, NanoServiceErrorStatus};
use to_do_dal::json_file::get_all as get_all_handle;

pub async fn get_all() -> Result<AllToDoItems, NanoServiceError> {
    Ok(AllToDoItems::from_hashmap(get_all_handle::<ToDoItem>()?))
}

pub async fn get_by_name(name: &str) -> Result<ToDoItem, NanoServiceError> {
    Ok(get_all_handle::<ToDoItem>()?
        .remove(name)
        .ok_or(NanoServiceError::new(
            format!("Item with name {} not found", name),
            NanoServiceErrorStatus::NotFound,
        ))?)
}
