use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub mod ser;
pub mod de;


pub enum WolframError {}
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
