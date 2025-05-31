use crate::error::PixuiError;

pub type PixuiResult<T> = Result<T, PixuiError>;

#[cfg(test)]
mod tests {
    use crate::context;
    use crate::result::PixuiResult;

    #[test]
    fn test_context_macro_ok() {
        let _result = {
            context!("grok stuff for {}", "bar" => {
                Ok(0)
            })
        }
        .unwrap();
    }

    #[test]
    fn test_context_macro_err() {
        fn my_broken_function() -> PixuiResult<u32> {
            Err("ungrokkable")?
        }
        let result = {
            context!("grok stuff for {}", "bar" => {
                my_broken_function()
            })
        }
        .expect_err("Should have errored, but was");
        assert_eq!(
            "General Error: Failed to grok stuff for bar",
            result.to_string()
        );
        assert!(format!("{:?}", result).contains("my_broken_function"));
    }
}
