use std::error::Error;
use std::fmt::Formatter;

#[derive(Debug)]
pub struct AppError {
    pub message: String
    //ToDo: Store cause
}

impl AppError {

    pub fn new(message: String) -> AppError {
        AppError {
            message
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AppError: {}", self.message)
    }
}

impl Error for AppError {}
