use std::fmt::Display;

use serde::de::Error;

use crate::serde_support::WolframError;

impl Error for WolframError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::RuntimeError {
            message: msg.to_string(),
        }
    }
}
