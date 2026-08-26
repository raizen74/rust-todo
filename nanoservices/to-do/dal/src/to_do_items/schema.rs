// file: nanoservices/to_do/dal/src/to_do_items/schema.rs
use std::fmt;
use serde::{Serialize, Deserialize};
use super::enums::TaskStatus;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewToDoItem {
    pub title: String,
    pub status: TaskStatus,
}

impl fmt::Display for NewToDoItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.status {
            TaskStatus::PENDING => write!(f, "Pending: {}", self.title),
            TaskStatus::DONE => write!(f, "Done: {}", self.title),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "sqlx-postgres", derive(sqlx::FromRow))]
pub struct ToDoItem {
    pub id: i32,
    pub title: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AllToDoItems {
    pub pending: Vec<ToDoItem>,
    pub done: Vec<ToDoItem>,
}

impl AllToDoItems {
    pub fn from_vec(all_items: Vec<ToDoItem>) -> AllToDoItems {
        let mut pending = Vec::new();
        let mut done = Vec::new();
        for ToDoItem { id, title, status } in all_items {
            if status == "PENDING" {
                pending.push(ToDoItem { id, title, status });
            } else if status == "DONE" {
                done.push(ToDoItem { id, title, status });
            }
        }
        AllToDoItems { pending, done }
    }
}
