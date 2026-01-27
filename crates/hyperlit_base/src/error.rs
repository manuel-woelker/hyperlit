use std::error::Error as StdError;
use std::fmt;
use std::path::PathBuf;

/* 📖 # Why a custom error type and not use anyhow/eyre/thiserror etc?

- Better control over error handling
- No dependencies to compile and integrate
- More transparency into error handling logic
 */

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

/* 📖 # Why use Box<Error> in the result type?

Boxing the error reduces the size of the result type, making it more efficient to return in the common case.

*/

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

impl<T> ResultExt<T> for Result<T> {
    fn context(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|err| Box::new(err.context(context)))
    }

    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|err| Box::new(err.with_context(f)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_error_from_file_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let path = PathBuf::from("test.txt");
        let kind = ErrorKind::FileError {
            path: path.clone(),
            source: io_err,
        };
        let error = Error::new(kind);

        match error.kind() {
            ErrorKind::FileError { path: p, .. } => {
                assert_eq!(p, &path);
            }
            _ => panic!("Expected FileError variant"),
        }
    }

    #[test]
    fn test_error_from_message() {
        let kind = ErrorKind::Message {
            message: "something went wrong".to_string(),
        };
        let error = Error::new(kind);

        match error.kind() {
            ErrorKind::Message { message } => {
                assert_eq!(message, "something went wrong");
            }
            _ => panic!("Expected Message variant"),
        }
    }

    #[test]
    fn test_error_context_attachment() {
        let kind = ErrorKind::Message {
            message: "original error".to_string(),
        };
        let error = Error::new(kind)
            .context("first context")
            .context("second context");

        assert_eq!(error.context.len(), 2);
        assert_eq!(error.context[0], "first context");
        assert_eq!(error.context[1], "second context");
    }

    #[test]
    fn test_error_with_context_lazy_evaluation() {
        let kind = ErrorKind::Message {
            message: "error".to_string(),
        };
        let mut called = false;
        let error = Error::new(kind).with_context(|| {
            called = true;
            "lazy context".to_string()
        });

        assert!(called);
        assert_eq!(error.context[0], "lazy context");
    }

    #[test]
    fn test_error_display_message_only() {
        let kind = ErrorKind::Message {
            message: "test message".to_string(),
        };
        let error = Error::new(kind);
        assert_eq!(error.to_string(), "test message");
    }

    #[test]
    fn test_error_display_with_context() {
        let kind = ErrorKind::Message {
            message: "test message".to_string(),
        };
        let error = Error::new(kind).context("operation failed");
        assert_eq!(error.to_string(), "operation failed: test message");
    }

    #[test]
    fn test_error_display_with_multiple_contexts() {
        let kind = ErrorKind::Message {
            message: "root error".to_string(),
        };
        let error = Error::new(kind)
            .context("first")
            .context("second")
            .context("third");
        assert_eq!(error.to_string(), "first: second: third: root error");
    }

    #[test]
    fn test_error_display_file_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "not found");
        let path = PathBuf::from("/tmp/test.txt");
        let kind = ErrorKind::FileError {
            path: path.clone(),
            source: io_err,
        };
        let error = Error::new(kind);
        let display = error.to_string();
        assert!(display.contains("/tmp/test.txt"));
        assert!(display.contains("not found"));
    }

    #[test]
    fn test_error_display_multiple_errors() {
        let msg1 = Error::new(ErrorKind::Message {
            message: "error 1".to_string(),
        });
        let msg2 = Error::new(ErrorKind::Message {
            message: "error 2".to_string(),
        });
        let kind = ErrorKind::Multiple {
            errors: vec![msg1, msg2],
            count: 2,
        };
        let error = Error::new(kind);
        let display = error.to_string();
        assert!(display.contains("Multiple errors occurred (2 total)"));
    }

    #[test]
    fn test_error_from_impl() {
        let kind = ErrorKind::Message {
            message: "test".to_string(),
        };
        let error: Error = kind.into();
        match error.kind() {
            ErrorKind::Message { message } => {
                assert_eq!(message, "test");
            }
            _ => panic!("Expected Message variant"),
        }
    }

    #[test]
    fn test_error_source_file_error() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let kind = ErrorKind::FileError {
            path: PathBuf::from("test.txt"),
            source: io_err,
        };
        let error = Error::new(kind);
        assert!(error.source().is_some());
    }

    #[test]
    fn test_error_source_message() {
        let kind = ErrorKind::Message {
            message: "test".to_string(),
        };
        let error = Error::new(kind);
        assert!(error.source().is_none());
    }

    #[test]
    fn test_error_source_multiple() {
        let msg = Error::new(ErrorKind::Message {
            message: "inner".to_string(),
        });
        let kind = ErrorKind::Multiple {
            errors: vec![msg],
            count: 1,
        };
        let error = Error::new(kind);
        assert!(error.source().is_none()); // Message has no source
    }

    #[test]
    fn test_error_root_cause_file_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "not found");
        let kind = ErrorKind::FileError {
            path: PathBuf::from("test.txt"),
            source: io_err,
        };
        let error = Error::new(kind);
        let root = error.root_cause();
        // The root cause is the io::Error itself
        assert_eq!(root.to_string(), "not found");
    }

    #[test]
    fn test_error_root_cause_message() {
        let kind = ErrorKind::Message {
            message: "test".to_string(),
        };
        let error = Error::new(kind);
        let root = error.root_cause();
        // For Message variant with no source, the root cause is the Error itself
        assert_eq!(root.to_string(), "test");
    }

    #[test]
    fn test_result_ext_context_success() {
        let result: Result<i32> = Ok(42);
        let final_result = result.context("operation failed");
        assert_eq!(final_result.unwrap(), 42);
    }

    #[test]
    fn test_result_ext_context_error() {
        let result: Result<i32> = Err(Box::new(Error::new(ErrorKind::Message {
            message: "original".to_string(),
        })));
        let final_result = result.context("operation failed");
        assert!(final_result.is_err());
        let err = final_result.unwrap_err();
        assert_eq!(err.to_string(), "operation failed: original");
    }

    #[test]
    fn test_result_ext_with_context_success() {
        let result: Result<i32> = Ok(42);
        let final_result = result.with_context(|| "operation failed".to_string());
        assert_eq!(final_result.unwrap(), 42);
    }

    #[test]
    fn test_result_ext_with_context_error() {
        let result: Result<i32> = Err(Box::new(Error::new(ErrorKind::Message {
            message: "original".to_string(),
        })));
        let final_result = result.with_context(|| "lazy context".to_string());
        assert!(final_result.is_err());
        let err = final_result.unwrap_err();
        assert_eq!(err.to_string(), "lazy context: original");
    }

    #[test]
    fn test_result_ext_chaining() {
        let result: Result<i32> = Err(Box::new(Error::new(ErrorKind::Message {
            message: "root".to_string(),
        })));
        let final_result = result
            .context("step 1")
            .context("step 2")
            .with_context(|| "step 3".to_string());
        assert!(final_result.is_err());
        let err = final_result.unwrap_err();
        assert_eq!(err.to_string(), "step 1: step 2: step 3: root");
    }

    #[test]
    fn test_multiple_errors_count() {
        let errors = vec![
            Error::new(ErrorKind::Message {
                message: "error 1".to_string(),
            }),
            Error::new(ErrorKind::Message {
                message: "error 2".to_string(),
            }),
        ];
        let kind = ErrorKind::Multiple { errors, count: 2 };
        let error = Error::new(kind);
        match error.kind() {
            ErrorKind::Multiple { count, .. } => {
                assert_eq!(count, &2);
            }
            _ => panic!("Expected Multiple variant"),
        }
    }
}
