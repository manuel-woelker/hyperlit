use std::error::Error as StdError;
use std::fmt;
use std::path::PathBuf;
use tracing_error::SpanTrace;
/* 📖 # Why a custom error type and not use anyhow/eyre/thiserror etc.?

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
    Multiple {
        errors: Vec<HyperlitError>,
        count: usize,
    },

    /// Wrapping std error
    StdError { error: Box<dyn std::error::Error> },

    /// Catch-all for other errors with a message
    Message { message: String },
}

/* 📖 # Why separate ErrorKind and HyperlitError?
This two-layer design provides a clear separation of concerns:
- ErrorKind: structural variants with specific contexts (file paths, line numbers, etc.)
- HyperlitError: wraps ErrorKind with additional runtime context string

Benefits:
- Users can pattern match on ErrorKind for specific handling
- HyperlitError provides ergonomic context attachment for propagation
- Avoids nested context strings (which get expensive with many layers)
- Aligns with Rust error handling best practices
*/

/// Comprehensive error type wrapping ErrorKind with optional context.
/// HyperlitError implements the standard Error trait and supports context attachment.
pub struct HyperlitError {
    kind: ErrorKind,
    span_trace: SpanTrace,
    context: Vec<String>,
    cause: Option<Box<HyperlitError>>,
}

impl HyperlitError {
    /// Creates a new error from an ErrorKind.
    pub fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            span_trace: SpanTrace::capture(),
            context: vec![],
            cause: None,
        }
    }

    pub fn message(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Message {
            message: message.into(),
        })
    }

    /// Attaches context to an error.
    /// Context is displayed before the error message.
    pub fn context(mut self, context: impl Into<String>) -> Self {
        self.context.push(context.into());
        self
    }

    pub fn get_context(&self) -> &Vec<String> {
        &self.context
    }

    pub fn caused_by(mut self, cause: impl Into<HyperlitError>) -> Self {
        self.cause = Some(Box::new(cause.into()));
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
    pub fn root_cause(&self) -> &HyperlitError {
        let mut current: &HyperlitError = self;
        while let Some(next) = &current.cause {
            current = next;
        }
        current
    }

    /// Returns the error cause.
    /// Returns None if there is no cause.
    pub fn get_cause(&self) -> Option<&HyperlitError> {
        self.cause.as_deref()
    }
}

impl From<ErrorKind> for HyperlitError {
    fn from(kind: ErrorKind) -> Self {
        Self::new(kind)
    }
}
/*
impl StdError for HyperlitError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &self.kind {
            ErrorKind::FileError { source, .. } => Some(source),
            ErrorKind::Multiple { errors, .. } => errors.first().and_then(|e| e.source()),
            ErrorKind::Message { .. } => None,
        }
    }
}
*/
impl fmt::Display for HyperlitError {
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
            ErrorKind::StdError { error } => {
                write!(f, "{}", error)
            }
        }
    }
}

/* 📖 # Why implement Debug manually for HyperlitError?
A custom Debug implementation provides:
- Readable, structured output of error context and spans using Unicode tree symbols
- Better diagnostics during debugging with visual hierarchy
- Snapshot testing friendly format with expect_test
- Alignment with error handling best practices (similar to eyre/anyhow)
*/

impl fmt::Debug for HyperlitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.format_debug(f)
    }
}

impl HyperlitError {
    /// Format the error with an optional indent prefix for nested display.
    /// Only the outermost error displays the span trace.
    fn format_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display the error message based on kind
        match &self.kind {
            ErrorKind::FileError { path, source } => {
                write!(f, "FileError: {}: {}", path.display(), source)?;
            }
            ErrorKind::Multiple { count, .. } => {
                write!(f, "Multiple errors ({} total)", count)?;
            }
            ErrorKind::Message { message } => {
                write!(f, "{}", message)?;
            }
            ErrorKind::StdError { error } => {
                write!(f, "{}", error)?;
            }
        }
        writeln!(f)?;

        // Display context and causes using the helper function
        self.format_context_and_causes(f, "")?;

        // Display span trace (only for outermost error)
        let span_trace = self.span_trace.to_string();
        if !span_trace.is_empty() {
            writeln!(f, "Trace: {}", span_trace)?;
        }

        Ok(())
    }

    /// Helper function to format context and error causes.
    /// This extracts the common logic shared between format_debug and format_debug_content.
    fn format_context_and_causes(&self, f: &mut fmt::Formatter<'_>, indent: &str) -> fmt::Result {
        // Determine what fields we have to display
        let has_context = !self.context.is_empty();
        let has_causes = self.cause.is_some();

        // Display context
        if has_context {
            for (i, ctx) in self.context.iter().enumerate() {
                let has_items_after = (i + 1 < self.context.len()) || has_causes;
                let prefix = if has_items_after { "├─" } else { "└─" };
                writeln!(f, "{}{} {}", indent, prefix, ctx)?;
            }
        }

        // Display error cause chain
        if has_causes {
            // Handle nested HyperlitError in cause field
            if let Some(nested_error) = &self.cause {
                // Print the cause prefix and nested error message on the same line
                match &nested_error.kind {
                    ErrorKind::FileError { path, source } => {
                        writeln!(
                            f,
                            "{}└─ cause: FileError: {}: {}",
                            indent,
                            path.display(),
                            source
                        )?;
                    }
                    ErrorKind::Multiple { count, .. } => {
                        writeln!(f, "{}└─ cause: Multiple errors ({} total)", indent, count)?;
                    }
                    ErrorKind::Message { message } => {
                        writeln!(f, "{}└─ cause: {}", indent, message)?;
                    }
                    ErrorKind::StdError { error } => {
                        writeln!(f, "{}└─ cause: {}", indent, error)?;
                    }
                }

                // Now format the nested error's content with proper indentation
                // Since cause is the last item (└─), use spaces instead of pipe for continuation
                let nested_indent = format!("{}   ", indent);
                nested_error.format_context_and_causes(f, &nested_indent)?;
            }
        }

        Ok(())
    }
}

/* 📖 # Why use Box<HyperlitError> in the result type?

Boxing the error reduces the size of the result type, making it more efficient to return in the common case.

*/

/// Standard result type for hyperlit_base operations.
pub type HyperlitResult<T> = Result<T, Box<HyperlitError>>;

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
    fn context(self, context: impl Into<String>) -> HyperlitResult<T>;

    /// Attaches context using lazy evaluation.
    /// Context is only evaluated if the result is an error.
    /// Prefer this to avoid expensive string formatting in the success path.
    fn with_context<F>(self, f: F) -> HyperlitResult<T>
    where
        F: FnOnce() -> String;
}

impl<T> ResultExt<T> for HyperlitResult<T> {
    fn context(self, context: impl Into<String>) -> HyperlitResult<T> {
        self.map_err(|err| Box::new(err.context(context)))
    }

    fn with_context<F>(self, f: F) -> HyperlitResult<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|err| Box::new(err.with_context(f)))
    }
}

impl<T: StdError + 'static> From<T> for Box<HyperlitError> {
    fn from(error: T) -> Self {
        Box::new(HyperlitError::new(ErrorKind::StdError {
            error: Box::new(error),
        }))
    }
}

/// Macro to bail out of a function with a formatted error message.
/// Similar to anyhow::bail! or eyre::bail!, this macro returns an error
/// with a formatted message using the standard format! syntax.
///
/// # Examples
///
/// ```rust,ignore
/// bail!("something went wrong");
/// bail!("expected {}, found {}", expected, actual);
/// ```
#[macro_export]
macro_rules! bail {
    ($msg:literal $(,)?) => {
        return Err(Box::new($crate::error::HyperlitError::message($msg)))
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err(Box::new($crate::error::HyperlitError::message(format!($fmt, $($arg)*))))
    };
}

/// Macro to create an error value with a formatted message.
/// Unlike `bail!`, this macro only creates the error value without returning.
/// Useful in closures where `return` would exit the closure instead of the function.
///
/// # Examples
///
/// ```rust,ignore
/// .map_err(|e| err!("parsing failed: {}", e))
/// .ok_or_else(|| err!("value not found"))
/// ```
#[macro_export]
macro_rules! err {
    ($msg:literal $(,)?) => {
        Box::new($crate::error::HyperlitError::message($msg))
    };
    ($fmt:expr, $($arg:tt)*) => {
        Box::new($crate::error::HyperlitError::message(format!($fmt, $($arg)*)))
    };
}
