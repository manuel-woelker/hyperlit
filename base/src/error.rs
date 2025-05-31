use error_stack::Report;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(thiserror::Error, Debug)]
pub enum HyperlitErrorKind {
    #[error("General Error: {0}")]
    General(String),

    #[error("Generic Error: {0}")]
    Generic(Box<dyn Error + Send + Sync + 'static>),

}

#[derive(Debug)]
pub struct HyperlitError(pub Report<HyperlitErrorKind>);

impl Display for HyperlitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl HyperlitError {
    #[track_caller]
    pub fn change_context<S: Into<String>>(self, message: S) -> Self {
        Self(
            self.0
                .change_context(HyperlitErrorKind::General(message.into())),
        )
    }
}

impl HyperlitError {
    #[track_caller]
    pub fn new(error: HyperlitErrorKind) -> HyperlitError {
        HyperlitError(Report::new(error))
    }
}

impl <T: Error + Send + Sync + 'static> From<T> for HyperlitError {
    #[track_caller]
    fn from(error: T) -> Self {
        let kind = HyperlitErrorKind::Generic(Box::new(error));
        let report = Report::new(kind);
        Self(report)
    }
}

#[macro_export]
macro_rules! bail {
    ($($args:tt)+) => {
        return Err($crate::error::HyperlitError::new($crate::error::HyperlitErrorKind::General(format!($($args)+).into())))
    }
}

#[macro_export]
macro_rules! err {
    ($($args:tt)+) => {
        $crate::error::HyperlitError::new($crate::error::HyperlitErrorKind::General(format!($($args)+).into()))
    };
}