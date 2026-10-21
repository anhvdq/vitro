use std::fmt;

/// How the worker loop reacts to a failed handler.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorClass {
    /// Transient (a timeout, a lost connection): retry with backoff until attempts run out.
    Retryable,
    /// Retrying cannot succeed (invalid input): bury at once.
    Permanent,
    /// A wiring or logic bug: bury at once so it is seen.
    Programming,
}

/// The one error type that crosses a port. Adapters translate library errors into it.
#[derive(Debug)]
pub struct PlatformError {
    class: ErrorClass,
    message: String,
}

impl PlatformError {
    pub fn retryable(message: impl Into<String>) -> Self {
        Self {
            class: ErrorClass::Retryable,
            message: message.into(),
        }
    }

    pub fn permanent(message: impl Into<String>) -> Self {
        Self {
            class: ErrorClass::Permanent,
            message: message.into(),
        }
    }

    pub fn programming(message: impl Into<String>) -> Self {
        Self {
            class: ErrorClass::Programming,
            message: message.into(),
        }
    }

    pub fn class(&self) -> ErrorClass {
        self.class
    }
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for PlatformError {}
