use chrono::NaiveDate;
use thiserror::Error;
use winnow::combinator::cond;

#[derive(Error, Debug)]
enum Error {}

#[derive(Debug, PartialEq)]
pub struct Task {
    pub(crate) deadline: NaiveDate,
    pub(crate) content: String,
}

impl Task {
    pub fn new(deadline: NaiveDate, content: &str) -> Self {
        Task {
            deadline,
            content: content.to_string(),
        }
    }
}
