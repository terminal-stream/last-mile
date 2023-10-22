use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;

#[derive(Debug)]
pub struct AppError {
    msg: String,
}

impl AppError {
    pub fn msg_string(msg: String) -> Self {
        AppError {
            msg
        }
    }
    pub fn msg_str(msg: &str) -> Self {
        AppError {
            msg: String::from(msg)
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
        AppError::msg_string(value.to_string())
    }
}