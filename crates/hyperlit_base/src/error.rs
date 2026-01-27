use std::error::Error as StdError;
use std::fmt;
use std::path::PathBuf;

/// Error variants that can occur in hyperlit operations.
/// Each variant represents a specific error category with its associated context.
#[derive(Debug)]
pub enum ErrorKind {
    /// File system operation failed
    FileError {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Multiple errors occurred during batch operations
    Multiple { errors: Vec<Error>, count: usize },

    /// Catch-all for other errors with a message
    Message { message: String },
}

/* 📖 # Why separate ErrorKind and Error?
This two-layer design provides a clear separation of concerns:
- ErrorKind: structural variants with specific contexts (file paths, line numbers, etc.)
- Error: wraps ErrorKind with additional runtime context string

Benefits:
- Users can pattern match on ErrorKind for specific handling
- Error provides ergonomic context attachment for propagation
- Avoids nested context strings (which get expensive with many layers)
- Aligns with Rust error handling best practices
*/

/// Comprehensive error type wrapping ErrorKind with optional context.
/// Error implements the standard Error trait and supports context attachment.
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    context: Vec<String>,
}

impl Error {
    /// Creates a new error from an ErrorKind.
    pub fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            context: vec![],
        }
    }

    /// Attaches context to an error.
    /// Context is displayed before the error message.
    pub fn context(mut self, context: impl Into<String>) -> Self {
        self.context.push(context.into());
        self
    }

    /// Attaches context using lazy evaluation.
    /// Useful to avoid expensive string construction for successful paths.
    pub fn with_context<F>(mut self, f: F) -> Self
    where
        F: FnOnce() -> String,
    {
        self.context.push(f());
        self
    }

    /// Returns a reference to the underlying ErrorKind.
    /// Allows pattern matching on specific error variants.
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// Returns the innermost error in the chain.
    /// Traverses the error source chain to find the root cause.
    pub fn root_cause(&self) -> &(dyn StdError + 'static) {
        let mut current: &(dyn StdError + 'static) = self;
        while let Some(next) = current.source() {
            current = next;
        }
        current
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self::new(kind)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &self.kind {
            ErrorKind::FileError { source, .. } => Some(source),
            ErrorKind::Multiple { errors, .. } => errors.first().and_then(|e| e.source()),
            ErrorKind::Message { .. } => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display context first if present
        for (i, ctx) in self.context.iter().enumerate() {
            if i == 0 {
                write!(f, "{}", ctx)?;
            } else {
                write!(f, ": {}", ctx)?;
            }
        }

        // Add a separator if we have context
        if !self.context.is_empty() {
            write!(f, ": ")?;
        }

        // Display the underlying error kind
        match &self.kind {
            ErrorKind::FileError { path, source } => {
                write!(f, "File error at {}: {}", path.display(), source)
            }
            ErrorKind::Multiple { errors, count } => {
                write!(
                    f,
                    "Multiple errors occurred ({} total): {}",
                    count,
                    errors.first().unwrap_or(self)
                )
            }
            ErrorKind::Message { message } => {
                write!(f, "{}", message)
            }
        }
    }
}

/// Standard result type for hyperlit_base operations.
pub type Result<T> = std::result::Result<T, Box<Error>>;

/* 📖 # Why provide ResultExt for context attachment?
The ResultExt trait provides ergonomic methods to add context to errors during propagation.
Using `.context("message")` is more readable than manually mapping and wrapping errors.
This pattern is common in error-handling libraries (e.g., anyhow, eyre).
*/

/// Extension trait for attaching context to Results.
/// Provides ergonomic error context attachment during error propagation.
pub trait ResultExt<T> {
    /// Attaches context to an error, consuming and re-wrapping it.
    /// Eager evaluation: context is evaluated immediately.
    fn context(self, context: impl Into<String>) -> Result<T>;

    /// Attaches context using lazy evaluation.
    /// Context is only evaluated if the result is an error.
    /// Prefer this to avoid expensive string formatting in the success path.
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;
}
