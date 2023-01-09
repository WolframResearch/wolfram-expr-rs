use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub mod de;
pub mod ser;


pub enum WolframError {
    RuntimeError { message: String },
}

impl Error for WolframError {}

impl Debug for WolframError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Display for WolframError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
