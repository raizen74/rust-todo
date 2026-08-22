use crate::enums::TaskStatus;
use crate::structs::ToDoItem;
use glue::errors::NanoServiceError;
// We define the feature because if we add another save_one function under a different storage engine,
// we will want our core to just work with the new feature
#[cfg(feature = "json-file-storage")]
use to_do_dal::json_file::save_one;

pub fn create(title: &str, status: TaskStatus) -> Result<ToDoItem, NanoServiceError> {
    let item = ToDoItem {
        title: title.to_string(),
        status,
    };
    let _ = save_one(&title.to_string(), &item)?;
    Ok(item)
}
