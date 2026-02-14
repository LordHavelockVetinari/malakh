use core::fmt;
use std::fmt::Display;

use crate::parse::location::Location;

#[derive(Debug, thiserror::Error)]
struct CompilationErrorInner {
    pub message: String,
    pub location: Location,
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct CompilationError(Box<CompilationErrorInner>);

impl CompilationError {
    pub fn new<S: Into<String>>(message: S, location: &Location) -> Self {
        Self(Box::new(CompilationErrorInner {
            message: message.into(),
            location: location.clone(),
        }))
    }

    pub fn err<S: Into<String>, R>(message: S, location: &Location) -> Result<R, Self> {
        Err(Self::new(message, location))
    }
}

impl Display for CompilationErrorInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.location.is_unknown() {
            write!(f, "compilation error: {}", self.message)
        } else {
            write!(
                f,
                "compilation error at {}: {}",
                self.location, self.message
            )
        }
    }
}
