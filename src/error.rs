use std::{
    error::Error,
    fmt::{Display, Formatter},
};

#[derive(Debug)]
pub enum PlotError {
    MissingAesthetic(String),
    InvalidAestheticType(String),
    ScaleError(String),
    ThemeError(String),
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
        }
    }
}

impl Error for PlotError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
