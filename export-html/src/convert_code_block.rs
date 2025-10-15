use crate::convert_tag::{ConversionContext, ConvertTag};
use hyperlit_base::result::HyperlitResult;
use std::io::Write;

#[derive(Default)]
pub struct ConvertCodeBlock {}

impl ConvertCodeBlock {
    pub fn new() -> Self {
        Self {}
    }
}

impl ConvertTag for ConvertCodeBlock {
    fn emit_before(
        &self,
        write: &mut dyn Write,
        _context: &ConversionContext,
    ) -> HyperlitResult<()> {
        write!(write, "<pre><code>")?;
        Ok(())
    }

    fn emit_after(
        &self,
        write: &mut dyn Write,
        _context: &ConversionContext,
    ) -> HyperlitResult<()> {
        write!(write, "</code></pre>")?;
        Ok(())
    }
}
