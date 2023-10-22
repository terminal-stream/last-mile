use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;

#[derive(Debug)]
pub struct AppError {
    msg: String,
}

impl AppError {
    pub fn msg(msg: String) -> Self {
        AppError {
            msg
        }
    }
}

impl Error for AppError {}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "App Error: {}", self.msg)
    }
}

impl From<io::Error> for AppError {
    fn from(value: io::Error) -> Self {
        AppError::msg(value.to_string())
    }
}