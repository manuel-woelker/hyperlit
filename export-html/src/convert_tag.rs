use hyperlit_base::result::HyperlitResult;
use hyperlit_model::element::Element;
use std::io::Write;

pub struct ConversionContext<'a> {
    pub element: &'a Element,
}

impl<'a> ConversionContext<'a> {
    pub fn new(element: &'a Element) -> Self {
        Self { element }
    }
}

pub trait ConvertTag {
    fn emit_before(&self, write: &mut dyn Write, context: &ConversionContext)
    -> HyperlitResult<()>;
    fn emit_after(&self, write: &mut dyn Write, context: &ConversionContext) -> HyperlitResult<()>;
}
