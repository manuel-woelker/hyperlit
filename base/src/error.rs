use error_stack::Report;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::num::ParseFloatError;

#[derive(thiserror::Error, Debug)]
pub enum PixuiErrorKind {
    #[error("General Error: {0}")]
    General(String),
}

#[derive(Debug)]
pub struct PixuiError(pub Report<PixuiErrorKind>);

impl Display for PixuiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl PixuiError {
    #[track_caller]
    pub fn change_context<S: Into<String>>(self, message: S) -> Self {
        Self(
            self.0
                .change_context(PixuiErrorKind::General(message.into())),
        )
    }
}

impl PixuiError {
    #[track_caller]
    pub fn new(error: PixuiErrorKind) -> PixuiError {
        PixuiError(Report::new(error))
    }
}

impl<T> From<T> for PixuiError
where
    for<'a> &'a T: Into<PixuiErrorKind>,
    T: Error + Send + Sync + 'static,
{
    #[track_caller]
    fn from(error: T) -> Self {
        let kind: PixuiErrorKind = (&error).into();
        let report = Report::new(error);
        let report = report.change_context(kind);
        Self(report)
    }
}

impl From<String> for PixuiErrorKind {
    #[track_caller]
    fn from(error: String) -> Self {
        Self::General(error)
    }
}

impl From<&std::io::Error> for PixuiErrorKind {
    #[track_caller]
    fn from(error: &std::io::Error) -> Self {
        Self::General(error.to_string())
    }
}

impl From<&ParseFloatError> for PixuiErrorKind {
    #[track_caller]
    fn from(error: &ParseFloatError) -> Self {
        Self::General(format!("Failed to parse float value: {}", error))
    }
}

impl From<&str> for PixuiError {
    #[track_caller]
    fn from(error: &str) -> Self {
        Self(Report::new(PixuiErrorKind::General(error.to_string())))
    }
}

#[macro_export]
macro_rules! bail {
    ($($args:tt)+) => {
        return Err($crate::error::PixuiError::new($crate::error::PixuiErrorKind::General(format!($($args)+).into())))
    }
}

#[macro_export]
macro_rules! err {
    ($($args:tt)+) => {
        $crate::error::PixuiError::new($crate::error::PixuiErrorKind::General(format!($($args)+).into()))
    };
}

#[macro_export]
macro_rules! context {
    ($fmt:expr $(, $($args:expr),+)? => $block:block) => {
        {
            $block
        }.map_err(|e: $crate::error::PixuiError| e.change_context(format!(concat!("Failed to ",$fmt) $(, $($args)+)?)))
    };
}
pub use context;
