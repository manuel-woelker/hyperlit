use crate::text_edit::TextEdit;
use hyperlit_base::error::bail;
use hyperlit_base::result::HyperlitResult;
use hyperlit_pal::PalHandle;

pub struct LiveEngine {
    pal: PalHandle,
}

impl LiveEngine {
    pub fn new(pal: PalHandle) -> LiveEngine {
        LiveEngine { pal }
    }

    pub fn apply_edit(&self, text_edit: TextEdit) -> HyperlitResult<()> {
        // TODO: file locking
        let original_string = self.pal.read_file_to_string(&text_edit.path)?;
        let offset = text_edit.offset;
        let range = offset..offset + text_edit.expected_text.len();
        if range.end > original_string.len() {
            bail!(
                "Cannot apply text edit to file '{}' (at offset {}-{}): offset out of bounds, file length is {} bytes",
                text_edit.path,
                offset,
                range.end,
                original_string.len()
            );
        }
        if !original_string.is_char_boundary(range.start)
            || !original_string.is_char_boundary(range.end)
        {
            bail!(
                "Cannot apply text edit to file '{}' (at offset {}): edit range is not a char boundary",
                text_edit.path,
                offset,
            );
        }

        let current_text = &original_string[range.clone()];
        if current_text != text_edit.expected_text {
            bail!(
                "Cannot apply text edit to file '{}' (at offset {}):\n\texpected: '{}'\n\tactual:   '{}'",
                text_edit.path,
                offset,
                text_edit.expected_text,
                current_text
            );
        }
        let new_string = original_string[..offset].to_string()
            + &text_edit.new_text
            + &original_string[range.end..];
        let mut output_file = self.pal.create_file(&text_edit.path)?;
        output_file.write_all(new_string.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text_edit::TextEdit;
    use expect_test::expect;
    use hyperlit_pal_mock::PalMock;

    fn create_engine() -> (PalMock, LiveEngine) {
        let pal = PalMock::default();
        pal.set_file("foo.txt", "foobar");
        pal.set_file("astro.txt", "👨‍🚀");
        let engine = LiveEngine::new(PalHandle::new(pal.clone()));
        (pal, engine)
    }

    #[test]
    fn test_replace() -> HyperlitResult<()> {
        let (pal, engine) = create_engine();
        engine.apply_edit(TextEdit::new("foo.txt", 3, "bar", "buzz"))?;
        pal.verify_effects(expect![[r#"
            READ FILE: foo.txt
            CREATE FILE: foo.txt
            WRITE FILE: foo.txt -> foobuzz
        "#]]);
        Ok(())
    }

    #[test]
    fn test_mismatch() -> HyperlitResult<()> {
        let (_pal, engine) = create_engine();
        let error = engine
            .apply_edit(TextEdit::new("foo.txt", 3, "baz", "buzz"))
            .expect_err("error expected");
        expect![[r#"
            Cannot apply text edit to file 'foo.txt' (at offset 3):
            	expected: 'baz'
            	actual:   'bar'"#]]
        .assert_eq(&error.to_string());
        Ok(())
    }

    #[test]
    fn test_offset_out_of_bounds() -> HyperlitResult<()> {
        let (_pal, engine) = create_engine();
        let error = engine
            .apply_edit(TextEdit::new("foo.txt", 4, "baz", "buzz"))
            .expect_err("error expected");
        expect!["Cannot apply text edit to file 'foo.txt' (at offset 4-7): offset out of bounds, file length is 6 bytes"]
            .assert_eq(&error.to_string());
        Ok(())
    }

    #[test]
    fn test_offset_is_multibyte_char() -> HyperlitResult<()> {
        let (_pal, engine) = create_engine();
        let error = engine
            .apply_edit(TextEdit::new("astro.txt", 1, "baz", "buzz"))
            .expect_err("error expected");
        expect!["Cannot apply text edit to file 'astro.txt' (at offset 1): edit range is not a char boundary"]
            .assert_eq(&error.to_string());
        Ok(())
    }

    #[test]
    fn test_edit_end_is_multibyte_char() -> HyperlitResult<()> {
        let (_pal, engine) = create_engine();
        let error = engine
            .apply_edit(TextEdit::new("astro.txt", 0, "b", "buzz"))
            .expect_err("error expected");
        expect!["Cannot apply text edit to file 'astro.txt' (at offset 0): edit range is not a char boundary"]
        .assert_eq(&error.to_string());
        Ok(())
    }
}
