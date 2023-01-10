use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub mod de;
pub mod ser;

#[derive(Debug)]
pub enum WolframError {
    RuntimeError { message: String },
}

impl Error for WolframError {}

impl Display for WolframError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WolframError::RuntimeError { message } => {
                writeln!(f, "RuntimeError: {}", message)
            },
        }
    }
}
