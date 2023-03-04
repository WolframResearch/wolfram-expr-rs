use std::fmt::Display;

use serde::de::Error;

use crate::WolframError;

impl Error for WolframError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        WolframError::runtime_error(msg)
    }
}
