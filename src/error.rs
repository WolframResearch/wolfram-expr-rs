use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

/// An error that occurred while processing a Wolfram Language expression.
#[derive(Clone, Debug, Hash)]
pub struct WolframError {
    // The field size should not change when new variants are added
    kind: Box<WolframErrorKind>,
}

/// The kind of error that occurred.
#[derive(Clone, Debug, Hash)]
pub enum WolframErrorKind {
    /// A runtime error.
    RuntimeError {
        /// The error message.
        message: String,
    },
}

impl Error for WolframError {}

impl Display for WolframError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.kind, f)
    }
}

impl WolframError {
    /// Get a reference to the kind of the error.
    pub fn get_kind(&self) -> &WolframErrorKind {
        &self.kind
    }
    /// Get a mutable reference to the kind of the error.
    pub fn mut_kind(&mut self) -> &mut WolframErrorKind {
        &mut self.kind
    }
    /// Replace the kind of the error.
    pub fn with_kind(kind: WolframErrorKind) -> Self {
        Self {
            kind: Box::new(kind),
        }
    }


    /// Creates a new runtime error.
    ///
    /// # Arguments
    ///
    /// * `message`:
    ///
    /// returns: WolframError
    ///
    /// # Examples
    ///
    /// ```
    /// # use wolfram_expr::WolframError;
    /// WolframError::runtime_error("This is a runtime error");
    /// ```
    pub fn runtime_error<S: Display>(message: S) -> Self {
        Self {
            kind: Box::new(WolframErrorKind::RuntimeError {
                message: message.to_string(),
            }),
        }
    }
}
