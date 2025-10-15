use crate::convert_tag::{ConversionContext, ConvertTag};
use hyperlit_base::result::HyperlitResult;
use std::io::Write;

#[derive(Default)]
pub struct ConvertPassthru {}

impl ConvertPassthru {
    pub fn new() -> Self {
        Self {}
    }
}

impl ConvertTag for ConvertPassthru {
    fn emit_before(
        &self,
        _write: &mut dyn Write,
        _context: &ConversionContext,
    ) -> HyperlitResult<()> {
        Ok(())
    }

    fn emit_after(
        &self,
        _write: &mut dyn Write,
        _context: &ConversionContext,
    ) -> HyperlitResult<()> {
        Ok(())
    }
}
