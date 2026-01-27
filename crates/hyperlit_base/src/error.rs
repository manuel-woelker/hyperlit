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
    pub cause: Option<Box<HyperlitError>>,
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

impl From<ErrorKind> for HyperlitError {
    fn from(kind: ErrorKind) -> Self {
        Self::new(kind)
    }
}

impl StdError for HyperlitError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &self.kind {
            ErrorKind::FileError { source, .. } => Some(source),
            ErrorKind::Multiple { errors, .. } => errors.first().and_then(|e| e.source()),
            ErrorKind::Message { .. } => None,
        }
    }
}

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
        self.format_debug(f, "")
    }
}

impl HyperlitError {
    /// Format the error with an optional indent prefix for nested display.
    /// Only the outermost error displays the span trace.
    fn format_debug(&self, f: &mut fmt::Formatter<'_>, indent: &str) -> fmt::Result {
        // Check if this is the outermost error (no indent)
        let is_outermost = indent.is_empty();

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
        }
        writeln!(f)?;

        // Determine what fields we have to display
        let has_context = !self.context.is_empty();
        let has_causes = self.source().is_some() || self.cause.is_some();
        let has_span = is_outermost && !format!("{}", self.span_trace).is_empty();

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
            // Handle source errors from ErrorKind
            if let Some(source) = self.source() {
                let has_items_after = self.cause.is_some();
                let prefix = if has_items_after { "├─" } else { "└─" };
                writeln!(f, "{}{} cause: {}", indent, prefix, source)?;

                // Walk the source chain
                let mut current = source;
                while let Some(next) = current.source() {
                    let has_items_after = self.cause.is_some();
                    let prefix = if has_items_after { "├─" } else { "└─" };
                    writeln!(f, "{}{} cause: {}", indent, prefix, next)?;
                    current = next;
                }
            }

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
                }

                // Now format the nested error's context and causes with proper indentation
                // Since cause is the last item (└─), use spaces instead of pipe for continuation
                let nested_indent = format!("{}   ", indent);
                nested_error.format_debug_content(f, &nested_indent)?;
            }
        }

        // Display span trace (only for outermost error)
        if has_span {
            writeln!(f, "{}Trace:", indent)?;
            let span_display = format!("{}", self.span_trace);
            for line in span_display.lines() {
                writeln!(f, "{}   {}", indent, line)?;
            }
        }

        Ok(())
    }

    /// Format the content (context, causes) of an error without printing the message.
    /// Used for nested errors where the message is printed on the same line as the cause prefix.
    fn format_debug_content(&self, f: &mut fmt::Formatter<'_>, indent: &str) -> fmt::Result {
        // Determine what fields we have to display
        let has_context = !self.context.is_empty();
        let has_causes = self.source().is_some() || self.cause.is_some();

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
            // Handle source errors from ErrorKind
            if let Some(source) = self.source() {
                let has_items_after = self.cause.is_some();
                let prefix = if has_items_after { "├─" } else { "└─" };
                writeln!(f, "{}{} cause: {}", indent, prefix, source)?;

                // Walk the source chain
                let mut current = source;
                while let Some(next) = current.source() {
                    let has_items_after = self.cause.is_some();
                    let prefix = if has_items_after { "├─" } else { "└─" };
                    writeln!(f, "{}{} cause: {}", indent, prefix, next)?;
                    current = next;
                }
            }

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
                }

                // Now format the nested error's content with proper indentation
                // Since cause is the last item (└─), use spaces instead of pipe for continuation
                let nested_indent = format!("{}   ", indent);
                nested_error.format_debug_content(f, &nested_indent)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use std::io;
    use tracing::warn_span;

    #[test]
    fn test_error_from_file_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let path = PathBuf::from("test.txt");
        let kind = ErrorKind::FileError {
            path: path.clone(),
            source: io_err,
        };
        let error = HyperlitError::new(kind);

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
        let error = HyperlitError::new(kind);

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
        let error = HyperlitError::new(kind)
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
        let error = HyperlitError::new(kind).with_context(|| {
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
        let error = HyperlitError::new(kind);
        assert_eq!(error.to_string(), "test message");
    }

    #[test]
    fn test_error_display_with_context() {
        let kind = ErrorKind::Message {
            message: "test message".to_string(),
        };
        let error = HyperlitError::new(kind).context("operation failed");
        assert_eq!(error.to_string(), "operation failed: test message");
    }

    #[test]
    fn test_error_display_with_multiple_contexts() {
        let kind = ErrorKind::Message {
            message: "root error".to_string(),
        };
        let error = HyperlitError::new(kind)
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
        let error = HyperlitError::new(kind);
        let display = error.to_string();
        assert!(display.contains("/tmp/test.txt"));
        assert!(display.contains("not found"));
    }

    #[test]
    fn test_error_display_multiple_errors() {
        let msg1 = HyperlitError::new(ErrorKind::Message {
            message: "error 1".to_string(),
        });
        let msg2 = HyperlitError::new(ErrorKind::Message {
            message: "error 2".to_string(),
        });
        let kind = ErrorKind::Multiple {
            errors: vec![msg1, msg2],
            count: 2,
        };
        let error = HyperlitError::new(kind);
        let display = error.to_string();
        assert!(display.contains("Multiple errors occurred (2 total)"));
    }

    #[test]
    fn test_error_from_impl() {
        let kind = ErrorKind::Message {
            message: "test".to_string(),
        };
        let error: HyperlitError = kind.into();
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
        let error = HyperlitError::new(kind);
        assert!(error.source().is_some());
    }

    #[test]
    fn test_error_source_message() {
        let kind = ErrorKind::Message {
            message: "test".to_string(),
        };
        let error = HyperlitError::new(kind);
        assert!(error.source().is_none());
    }

    #[test]
    fn test_error_source_multiple() {
        let msg = HyperlitError::new(ErrorKind::Message {
            message: "inner".to_string(),
        });
        let kind = ErrorKind::Multiple {
            errors: vec![msg],
            count: 1,
        };
        let error = HyperlitError::new(kind);
        assert!(error.source().is_none()); // Message has no source
    }

    #[test]
    fn test_error_root_cause_file_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "not found");
        let kind = ErrorKind::FileError {
            path: PathBuf::from("test.txt"),
            source: io_err,
        };
        let error = HyperlitError::new(kind);
        let root = error.root_cause();
        // The root cause is the io::Error itself
        assert_eq!(root.to_string(), "not found");
    }

    #[test]
    fn test_error_root_cause_message() {
        let kind = ErrorKind::Message {
            message: "test".to_string(),
        };
        let error = HyperlitError::new(kind);
        let root = error.root_cause();
        // For Message variant with no source, the root cause is the Error itself
        assert_eq!(root.to_string(), "test");
    }

    #[test]
    fn test_result_ext_context_success() {
        let result: HyperlitResult<i32> = Ok(42);
        let final_result = result.context("operation failed");
        assert_eq!(final_result.unwrap(), 42);
    }

    #[test]
    fn test_result_ext_context_error() {
        let result: HyperlitResult<i32> = Err(Box::new(HyperlitError::new(ErrorKind::Message {
            message: "original".to_string(),
        })));
        let final_result = result.context("operation failed");
        assert!(final_result.is_err());
        let err = final_result.unwrap_err();
        assert_eq!(err.to_string(), "operation failed: original");
    }

    #[test]
    fn test_result_ext_with_context_success() {
        let result: HyperlitResult<i32> = Ok(42);
        let final_result = result.with_context(|| "operation failed".to_string());
        assert_eq!(final_result.unwrap(), 42);
    }

    #[test]
    fn test_result_ext_with_context_error() {
        let result: HyperlitResult<i32> = Err(Box::new(HyperlitError::new(ErrorKind::Message {
            message: "original".to_string(),
        })));
        let final_result = result.with_context(|| "lazy context".to_string());
        assert!(final_result.is_err());
        let err = final_result.unwrap_err();
        assert_eq!(err.to_string(), "lazy context: original");
    }

    #[test]
    fn test_result_ext_chaining() {
        let result: HyperlitResult<i32> = Err(Box::new(HyperlitError::new(ErrorKind::Message {
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
            HyperlitError::new(ErrorKind::Message {
                message: "error 1".to_string(),
            }),
            HyperlitError::new(ErrorKind::Message {
                message: "error 2".to_string(),
            }),
        ];
        let kind = ErrorKind::Multiple { errors, count: 2 };
        let error = HyperlitError::new(kind);
        match error.kind() {
            ErrorKind::Multiple { count, .. } => {
                assert_eq!(count, &2);
            }
            _ => panic!("Expected Multiple variant"),
        }
    }

    #[test]
    fn test_spantrace_captured_in_error() {
        use tracing::span;
        use tracing_error::ErrorLayer;
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;

        // 📖 # Why set up a subscriber in the test?
        // SpanTrace::capture() requires an active tracing subscriber to record span information.
        // Without initializing the subscriber, the span trace would be empty.
        // We use `try_init()` variant to handle multiple tests running concurrently.

        // Set up tracing with ErrorLayer
        let _ = tracing_subscriber::registry()
            .with(ErrorLayer::default())
            .try_init();

        // Create a nested span context
        let outer_span = span!(tracing::Level::DEBUG, "outer_operation");
        let _outer_guard = outer_span.enter();

        let inner_span = span!(tracing::Level::DEBUG, "inner_operation");
        let _inner_guard = inner_span.enter();

        // Create an error inside the spans
        let kind = ErrorKind::Message {
            message: "error in nested context".to_string(),
        };
        let error = HyperlitError::new(kind);

        // Verify that spantrace was captured (should not be empty when subscriber is active)
        let spantrace_display = format!("{:?}", error.span_trace);
        // The spantrace should contain some debug information indicating it was captured
        // When spans are active, the debug representation will show span context
        assert!(
            !spantrace_display.is_empty(),
            "SpanTrace should be captured"
        );

        let debug_output = format!("{:?}", error);
        expect![[r#"
            error in nested context
            Trace:
                  0: hyperlit_base::error::tests::inner_operation
                            at crates\hyperlit_base\src\error.rs:664
                  1: hyperlit_base::error::tests::outer_operation
                            at crates\hyperlit_base\src\error.rs:661
        "#]]
        .assert_eq(&debug_output);
    }

    #[test]
    fn test_spantrace_display_includes_span_information() {
        use tracing::span;
        use tracing_error::ErrorLayer;
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;

        let _ = tracing_subscriber::registry()
            .with(ErrorLayer::default())
            .try_init();

        // Create a specifically named span
        let operation_span = span!(tracing::Level::DEBUG, "test_operation", operation_id = 42);
        let _guard = operation_span.enter();

        let kind = ErrorKind::Message {
            message: "test error message".to_string(),
        };
        let error = HyperlitError::new(kind);

        expect![[r#"
            test error message
            Trace:
                  0: hyperlit_base::error::tests::test_operation
                          with operation_id=42
                            at crates\hyperlit_base\src\error.rs:706

        "#]]
        .assert_debug_eq(&error);
    }

    #[test]
    fn test_error_with_spantrace_and_context() {
        use tracing::span;
        use tracing_error::ErrorLayer;
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;

        let _ = tracing_subscriber::registry()
            .with(ErrorLayer::default())
            .try_init();

        let operation_span = span!(tracing::Level::INFO, "error_with_context");
        let _guard = operation_span.enter();

        let kind = ErrorKind::Message {
            message: "base error".to_string(),
        };
        let error = HyperlitError::new(kind)
            .context("operation failed")
            .with_context(|| "additional context".to_string());

        // Verify context is preserved
        assert_eq!(error.context.len(), 2);
        assert_eq!(error.context[0], "operation failed");
        assert_eq!(error.context[1], "additional context");

        expect![[r#"
            base error
            ├─ operation failed
            └─ additional context
            Trace:
                  0: hyperlit_base::error::tests::error_with_context
                            at crates\hyperlit_base\src\error.rs:736

        "#]]
        .assert_debug_eq(&error);
    }

    #[test]
    fn test_debug_pretty_print_format() {
        use tracing::span;
        use tracing_error::ErrorLayer;
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;

        // 📖 # Why test the Debug format explicitly?
        // The custom Debug impl provides human-readable output for error diagnostics.
        // This test verifies the structured format: message, context, source, and span trace.

        let _ = tracing_subscriber::registry()
            .with(ErrorLayer::default())
            .try_init();

        let operation_span = span!(tracing::Level::DEBUG, "operation");
        let _guard = operation_span.enter();

        let kind = ErrorKind::Message {
            message: "something went wrong".to_string(),
        };
        let error = HyperlitError::new(kind)
            .context("during file processing")
            .context("in batch job");

        expect![[r#"
            something went wrong
            ├─ during file processing
            └─ in batch job
            Trace:
                  0: hyperlit_base::error::tests::operation
                            at crates\hyperlit_base\src\error.rs:778

        "#]]
        .assert_debug_eq(&error);
    }

    #[test]
    fn test_debug_nested_errors() {
        use tracing::span;
        use tracing_error::ErrorLayer;
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;

        let _ = tracing_subscriber::registry()
            .with(ErrorLayer::default())
            .try_init();

        let operation_span = span!(tracing::Level::DEBUG, "operation");
        let _guard = operation_span.enter();

        let inner_error = HyperlitError::message("inner error").context("inner context");

        let outer_span = warn_span!("outer span");
        let _outer_guard = outer_span.enter();

        let mut outer_error = HyperlitError::message("outer error").context("outer context");
        outer_error.cause = Some(Box::new(inner_error));

        expect![[r#"
            outer error
            ├─ outer context
            └─ cause: inner error
               └─ inner context
            Trace:
                  0: hyperlit_base::error::tests::outer span
                            at crates\hyperlit_base\src\error.rs:816
                  1: hyperlit_base::error::tests::operation
                            at crates\hyperlit_base\src\error.rs:811

        "#]]
        .assert_debug_eq(&outer_error);
    }

    #[test]
    fn test_debug_multiple_nested_errors() {
        use tracing::span;
        use tracing_error::ErrorLayer;
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;

        let _ = tracing_subscriber::registry()
            .with(ErrorLayer::default())
            .try_init();

        let operation_span = span!(tracing::Level::DEBUG, "operation", foo = 42);
        let _guard = operation_span.enter();

        let error_1 = HyperlitError::message("error 1").context("context 1");

        let outer_span = warn_span!("outer span");
        let _outer_guard = outer_span.enter();

        let mut error_2 = HyperlitError::message("error 2").context("context 2");
        error_2.cause = Some(Box::new(error_1));

        let mut error_3 = HyperlitError::message("error 3").context("context 3");
        error_3.cause = Some(Box::new(error_2));

        expect![[r#"
            error 3
            ├─ context 3
            └─ cause: error 2
               ├─ context 2
               └─ cause: error 1
                  └─ context 1
            Trace:
                  0: hyperlit_base::error::tests::outer span
                            at crates\hyperlit_base\src\error.rs:853
                  1: hyperlit_base::error::tests::operation
                          with foo=42
                            at crates\hyperlit_base\src\error.rs:848

        "#]]
        .assert_debug_eq(&error_3);
    }
}
