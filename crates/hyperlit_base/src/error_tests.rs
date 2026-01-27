/* 📖 # Why use a separate file for these error tests?

The test cases verify span traces which contain line numbers.

To prevent these line numbers from changing when modifying the main error module, we use a separate file for the tests.
*/

#[cfg(test)]
mod tests {
    use crate::error::ErrorKind;
    use crate::{HyperlitError, HyperlitResult, ResultExt};
    use expect_test::expect;
    use std::error::Error;
    use std::io;
    use std::path::PathBuf;
    use tracing::span;
    use tracing::warn_span;
    use tracing_error::ErrorLayer;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    // 📖 # Why set up a subscriber in the test?
    // SpanTrace::capture() requires an active tracing subscriber to record span information.
    // Without initializing the subscriber, the span trace would be empty.
    // We use `try_init()` variant to handle multiple tests running concurrently.

    // Set up tracing with ErrorLayer

    /// Set up tracing with ErrorLayer for tests.
    /// Uses `try_init()` to handle multiple tests running concurrently.
    fn setup_tracing_subscriber() {
        let _ = tracing_subscriber::registry()
            .with(ErrorLayer::default())
            .try_init();
    }

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

        assert_eq!(error.get_context().len(), 2);
        assert_eq!(error.get_context()[0], "first context");
        assert_eq!(error.get_context()[1], "second context");
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
        assert_eq!(error.get_context()[0], "lazy context");
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
    fn test_spantrace_display_includes_span_information() {
        setup_tracing_subscriber();

        // Create a specifically named span
        let operation_span = span!(tracing::Level::DEBUG, "test_operation", operation_id = 42);
        let _guard = operation_span.enter();

        let kind = ErrorKind::Message {
            message: "test error message".to_string(),
        };
        let error = HyperlitError::new(kind);

        expect![[r#"
            test error message
            Trace:    0: hyperlit_base::error_tests::tests::test_operation
                       with operation_id=42
                         at crates\hyperlit_base\src\error_tests.rs:305

        "#]]
        .assert_debug_eq(&error);
    }

    #[test]
    fn test_error_with_spantrace_and_context() {
        setup_tracing_subscriber();

        let operation_span = span!(tracing::Level::INFO, "error_with_context");
        let _guard = operation_span.enter();

        let kind = ErrorKind::Message {
            message: "base error".to_string(),
        };
        let error = HyperlitError::new(kind)
            .context("operation failed")
            .with_context(|| "additional context".to_string());

        // Verify context is preserved
        assert_eq!(error.get_context().len(), 2);
        assert_eq!(error.get_context()[0], "operation failed");
        assert_eq!(error.get_context()[1], "additional context");

        expect![[r#"
            base error
            ├─ operation failed
            └─ additional context
            Trace:    0: hyperlit_base::error_tests::tests::error_with_context
                         at crates\hyperlit_base\src\error_tests.rs:327

        "#]]
        .assert_debug_eq(&error);
    }

    #[test]
    fn test_debug_pretty_print_format() {
        // 📖 # Why test the Debug format explicitly?
        // The custom Debug impl provides human-readable output for error diagnostics.
        // This test verifies the structured format: message, context, source, and span trace.

        setup_tracing_subscriber();

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
            Trace:    0: hyperlit_base::error_tests::tests::operation
                         at crates\hyperlit_base\src\error_tests.rs:361

        "#]]
        .assert_debug_eq(&error);
    }

    #[test]
    fn test_debug_nested_errors() {
        setup_tracing_subscriber();

        let operation_span = span!(tracing::Level::DEBUG, "operation");
        let _guard = operation_span.enter();

        let inner_error = HyperlitError::message("inner error").context("inner context");

        let outer_span = warn_span!("outer span");
        let _outer_guard = outer_span.enter();

        let outer_error = HyperlitError::message("outer error")
            .context("outer context")
            .caused_by(inner_error);

        expect![[r#"
            outer error
            ├─ outer context
            └─ cause: inner error
               └─ inner context
            Trace:    0: hyperlit_base::error_tests::tests::outer span
                         at crates\hyperlit_base\src\error_tests.rs:391
               1: hyperlit_base::error_tests::tests::operation
                         at crates\hyperlit_base\src\error_tests.rs:386

        "#]]
        .assert_debug_eq(&outer_error);
    }

    #[test]
    fn test_debug_multiple_nested_errors() {
        setup_tracing_subscriber();

        let operation_span = span!(tracing::Level::DEBUG, "operation", foo = 42);
        let _guard = operation_span.enter();

        let error_1 = HyperlitError::message("error 1").context("context 1");

        let outer_span = warn_span!("outer span");
        let _outer_guard = outer_span.enter();

        let error_2 = HyperlitError::message("error 2")
            .context("context 2")
            .caused_by(error_1);

        let error_3 = HyperlitError::message("error 3")
            .context("context 3")
            .caused_by(error_2);

        expect![[r#"
            error 3
            ├─ context 3
            └─ cause: error 2
               ├─ context 2
               └─ cause: error 1
                  └─ context 1
            Trace:    0: hyperlit_base::error_tests::tests::outer span
                         at crates\hyperlit_base\src\error_tests.rs:421
               1: hyperlit_base::error_tests::tests::operation
                       with foo=42
                         at crates\hyperlit_base\src\error_tests.rs:416

        "#]]
        .assert_debug_eq(&error_3);
    }
}
