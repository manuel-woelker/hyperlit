use crate::convert_tag::{ConversionContext, ConvertTag};
use hyperlit_base::result::HyperlitResult;
use hyperlit_model::attributes;
use hyperlit_model::value::Value;
use std::io::Write;

#[derive(Default)]
pub struct ConvertLink {}

impl ConvertLink {
    pub fn new() -> Self {
        Self {}
    }
}

impl ConvertTag for ConvertLink {
    fn emit_before(
        &self,
        write: &mut dyn Write,
        context: &ConversionContext,
    ) -> HyperlitResult<()> {
        if let Some(href) = context
            .element
            .get_attribute(attributes::LINK_DESTINATION_URL)
            .map(Value::as_string)
        {
            write!(write, "<a href=\"{}\">", href)?;
        } else {
            write!(write, "<a>")?;
        }
        Ok(())
    }

    fn emit_after(
        &self,
        write: &mut dyn Write,
        _context: &ConversionContext,
    ) -> HyperlitResult<()> {
        write!(write, "</a>")?;
        Ok(())
    }
}
