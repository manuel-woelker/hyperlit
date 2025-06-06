use error_stack::{Context, Report};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub struct HyperlitError(pub Report<HyperlitErrorContext>);

pub struct HyperlitErrorContext {
    pub context: Box<dyn Context>,
}

impl Debug for HyperlitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

impl HyperlitErrorContext {
    #[track_caller]
    pub fn new<T: Context>(context: T) -> HyperlitErrorContext {
        HyperlitErrorContext {
            context: Box::new(context),
        }
    }

    #[track_caller]
    pub fn from_string<T: Into<String>>(message: T) -> HyperlitErrorContext {
        HyperlitErrorContext {
            context: Box::new(GeneralError {
                message: message.into(),
            }),
        }
    }
}

pub struct BoxedErrorContext {
    pub error: BoxedError,
}

pub type BoxedError = Box<dyn Error + Send + Sync + 'static>;

impl BoxedErrorContext {
    #[track_caller]
    pub fn new(error: BoxedError) -> BoxedErrorContext {
        BoxedErrorContext { error }
    }
}

impl Display for BoxedErrorContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.error, f)
    }
}

impl Debug for BoxedErrorContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.error, f)
    }
}

impl Error for BoxedErrorContext {}

impl HyperlitErrorContext {
    pub fn from_context<T: Context>(context: T) -> HyperlitErrorContext {
        HyperlitErrorContext {
            context: Box::new(context),
        }
    }
}

impl Display for HyperlitErrorContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.context, f)
    }
}

impl Debug for HyperlitErrorContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.context, f)
    }
}

impl Error for HyperlitErrorContext {}

pub struct GeneralError {
    pub message: String,
}

impl Display for GeneralError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.message, f)
    }
}

impl Debug for GeneralError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.message, f)
    }
}

impl Error for GeneralError {}

impl Display for HyperlitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl HyperlitError {
    #[track_caller]
    pub fn change_context<S: Into<String>>(self, message: S) -> Self {
        Self(
            self.0
                .change_context(HyperlitErrorContext::from_string(message.into())),
        )
    }
}

impl HyperlitError {
    #[track_caller]
    pub fn new<T: Context>(error: T) -> HyperlitError {
        HyperlitError(Report::new(HyperlitErrorContext::from_context(error)))
    }

    #[track_caller]
    pub fn from_string<T: Into<String>>(message: T) -> HyperlitError {
        HyperlitError(Report::new(HyperlitErrorContext::from_string(message)))
    }

    #[track_caller]
    pub fn from_boxed(error: BoxedError) -> HyperlitErrorContext {
        HyperlitErrorContext {
            context: Box::new(BoxedErrorContext::new(error)),
        }
    }
}

impl<T: Context> From<T> for HyperlitError {
    #[track_caller]
    fn from(error: T) -> Self {
        Self::new(error)
    }
}

#[macro_export]
macro_rules! bail {
    ($($args:tt)+) => {
        return Err($crate::err!($($args)+))
    }
}

#[macro_export]
macro_rules! err {
    ($($args:tt)+) => {
        $crate::error::HyperlitError::from_string(format!($($args)+))
    };
}
