use std::{
    error::Error,
    fmt::{Display, Formatter},
};

pub type Result<T> = std::result::Result<T, PlotError>;

#[derive(Debug)]
pub enum PlotError {
    MissingAesthetic(String),
    InvalidAestheticType(String),
    ScaleError(String),
    ThemeError(String),
    InvalidData(String),
    Generic(String),
}

impl Display for PlotError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PlotError::MissingAesthetic(aes) => {
                write!(f, "Missing aesthetic: {}", aes)
            }
            PlotError::InvalidAestheticType(aes) => {
                write!(f, "Invalid type for aesthetic: {}", aes)
            }
            PlotError::ScaleError(msg) => {
                write!(f, "Scale error: {}", msg)
            }
            PlotError::ThemeError(msg) => {
                write!(f, "Theme error: {}", msg)
            }
            PlotError::InvalidData(msg) => {
                write!(f, "Invalid data: {}", msg)
            }
            PlotError::Generic(msg) => {
                write!(f, "Error: {}", msg)
            }
        }
    }
}

impl Error for PlotError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
