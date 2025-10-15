use crate::convert_tag::{ConversionContext, ConvertTag};
use hyperlit_base::result::HyperlitResult;
use std::io::Write;

#[derive(Default)]
pub struct ConvertHeading {}

impl ConvertHeading {
    pub fn new() -> Self {
        Self {}
    }

    fn get_level<'a>(context: &'a ConversionContext) -> &'a str {
        context
            .element
            .get_attribute("level")
            .map(|level| level.as_string())
            .unwrap_or("1")
    }
}

impl ConvertTag for ConvertHeading {
    fn emit_before(
        &self,
        write: &mut dyn Write,
        context: &ConversionContext,
    ) -> HyperlitResult<()> {
        let level = Self::get_level(context);
        write!(write, "<h{}>", level)?;
        Ok(())
    }

    fn emit_after(&self, write: &mut dyn Write, context: &ConversionContext) -> HyperlitResult<()> {
        let level = Self::get_level(context);
        write!(write, "</h{}>", level)?;
        Ok(())
    }
}
