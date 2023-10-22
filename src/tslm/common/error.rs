use std::error::Error;
use std::fmt::{Display, Formatter};

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
    pub fn msg_str(msg: &str) -> Self {
        AppError::msg(String::from(msg))
    }
    pub fn from(error: impl Error) -> Self {
        AppError::msg(error.to_string())
    }

}

impl Error for AppError {}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "App Error: {}", self.msg)
    }
}
