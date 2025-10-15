use crate::convert_tag::{ConversionContext, ConvertTag};
use hyperlit_base::result::HyperlitResult;
use std::borrow::Cow;
use std::io::Write;

pub const PICO_CSS: &str = include_str!("css/pico.classless.jade.css");

#[derive(Default)]
pub struct ConvertDocument {
    css: Option<Cow<'static, str>>,
}

impl ConvertDocument {
    pub fn new() -> Self {
        Self {
            css: Some(Cow::Borrowed(PICO_CSS)),
        }
    }

    pub fn new_inline_css(css: impl Into<Cow<'static, str>>) -> Self {
        Self {
            css: Some(css.into()),
        }
    }
}

impl ConvertTag for ConvertDocument {
    fn emit_before(
        &self,
        write: &mut dyn Write,
        _context: &ConversionContext,
    ) -> HyperlitResult<()> {
        writeln!(write, "<!DOCTYPE html>")?;
        writeln!(write, "<html>")?;
        writeln!(write, "<head>")?;
        if let Some(css) = &self.css {
            writeln!(write, "<style>")?;
            writeln!(write, "{}", css)?;
            writeln!(write, "</style>")?;
        }
        writeln!(write, "</head>")?;
        writeln!(write, "<body>")?;
        writeln!(write, "<main>")?;
        Ok(())
    }

    fn emit_after(
        &self,
        write: &mut dyn Write,
        _context: &ConversionContext,
    ) -> HyperlitResult<()> {
        writeln!(write, "</main>")?;
        writeln!(write, "</body>")?;
        writeln!(write, "</html>")?;
        Ok(())
    }
}
