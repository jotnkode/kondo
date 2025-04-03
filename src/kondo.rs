use chrono::NaiveDate;
use thiserror::Error;


#[derive(Error, Debug)]
enum Error {}

#[derive(Debug, PartialEq)]
pub struct Task {
    pub id: i64,
    pub deadline: NaiveDate,
    pub content: String,
    pub category: Option<String>,
    pub done: bool,
}

impl Task {
    pub fn new(id: Option<i64>, deadline: NaiveDate, content: &str) -> Self {
        Task {
            id: id.unwrap_or(0_i64),
            deadline,
            content: content.to_string(),
            category: None,
            done: false,
        }
    }
}
