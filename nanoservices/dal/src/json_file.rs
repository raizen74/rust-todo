use glue::errors::{NanoServiceError, NanoServiceErrorStatus};
use glue::safe_eject;
use serde::{Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

fn get_handle(path: Option<&str>) -> Result<File, NanoServiceError> {
    let path = match path {
        Some(p) => p,
        None => &env::var("JSON_STORE_PATH").unwrap_or("./tasks.json".to_string()),
    };
    safe_eject!(
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path),
        NanoServiceErrorStatus::Unknown,
        "Error writing tasks to JSON to file"
    )
}

fn get_write_handle(path: Option<&str>) -> Result<File, NanoServiceError> {
    let path = match path {
        Some(p) => p,
        None => &env::var("JSON_STORE_PATH").unwrap_or("./tasks.json".to_string()),
    };
    let file = safe_eject!(
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true) // ensures file is empty
            .open(&path),
        NanoServiceErrorStatus::Unknown,
        "Error reading JSON file (write handle)"
    )?;
    Ok(file)
}

pub fn get_all<T: DeserializeOwned>() -> Result<HashMap<String, T>, NanoServiceError> {
    let mut file = get_handle(None)?;
    let mut contents = String::new();
    safe_eject!(
        file.read_to_string(&mut contents),
        NanoServiceErrorStatus::Unknown,
        "Error reading JSON file to get all tasks"
    )?;
    let tasks: HashMap<String, T> = safe_eject!(
        serde_json::from_str(&contents),
        NanoServiceErrorStatus::Unknown,
        "Error parsing JSON file"
    )?;
    Ok(tasks)
}

pub fn save_all<T: Serialize>(tasks: &HashMap<String, T>) -> Result<(), NanoServiceError> {
    let mut file = get_write_handle(None)?;
    let json = safe_eject!(
        serde_json::to_string_pretty(tasks),
        NanoServiceErrorStatus::Unknown,
        "Error serializing JSON before saving tasks"
    )?;
    safe_eject!(
        file.write_all(json.as_bytes()),
        NanoServiceErrorStatus::Unknown,
        "Error writing tasks to JSON to file"
    )?;
    Ok(())
}

pub fn get_one<T: DeserializeOwned + Clone>(id: &str) -> Result<T, NanoServiceError> {
    let tasks = get_all::<T>()?;
    match tasks.get(id) {
        Some(t) => Ok(t.clone()),
        None => Err(NanoServiceError::new(
            format!("Task with id {} not found", id),
            NanoServiceErrorStatus::Unknown,
        )),
    }
}

pub fn save_one<T>(id: &str, task: &T) -> Result<(), NanoServiceError>
where
    T: Serialize + DeserializeOwned + Clone,
{
    let mut tasks = get_all::<T>().unwrap_or_else(|_| HashMap::new());
    tasks.insert(id.to_string(), task.clone());
    save_all(&tasks)
}

pub fn delete_one<T>(id: &str) -> Result<T, NanoServiceError>
where
    T: Serialize + DeserializeOwned + Clone + std::fmt::Debug,
{
    let mut tasks = get_all::<T>().unwrap_or(HashMap::new());
    match tasks.remove(id) {
        Some(deleted_item) => {
            save_all(&tasks)?;
            Ok(deleted_item)
        }
        None => Err(NanoServiceError::new(
            format!("Task with title {} not found", id),
            NanoServiceErrorStatus::NotFound,
        )),
    }
}
