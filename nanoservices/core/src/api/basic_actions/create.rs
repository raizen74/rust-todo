use crate::structs::ToDoItem;

// We define the feature because if we add another save_one function under a different storage engine,
// we will want our core to just work with the new feature
use glue::errors::NanoServiceError;
#[cfg(feature = "json-file-storage")]
use to_do_dal::json_file::save_one;

pub async fn create(item: ToDoItem) -> Result<ToDoItem, NanoServiceError> {
    let _ = save_one(&item.title.to_string(), &item)?;
    Ok(item)
}
