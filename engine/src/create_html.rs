use crate::engine::HyperlitEngine;
use hyperlit_base::result::HyperlitResult;
use hyperlit_pal::{FilePath, PalHandle};
use hyperlit_pal_real::PalReal;
use std::fs;

pub fn create_html() -> HyperlitResult<()> {
    let pal = PalHandle::new(PalReal::new());
    let engine = HyperlitEngine::new_handle(pal.clone());
    engine.init();
    let output_path = FilePath::from("output");
    pal.remove_directory_all(&output_path)?;
    pal.create_directory_all(&output_path)?;
    let book_html = engine.render_book_html()?;
    let title = engine.get_book_title()?;
    let mut index_html = format!(
        r#"
    <!DOCTYPE html>
<html>
<head>
    <title>{title}</title>
    <link rel="stylesheet" href="layout.css">
</head>
<body>
    "#
    );
    index_html += &book_html;
    index_html += "</body></html>\n";
    let mut index_file = pal.create_file(&output_path.join_normalized("index.html"))?;
    index_file.write_all(index_html.as_bytes())?;

    // Layout css
    let layout_css = fs::read_to_string("server/src/assets/live_service.css")?;
    let mut layout_css_file = pal.create_file(&output_path.join_normalized("layout.css"))?;
    layout_css_file.write_all(layout_css.as_bytes())?;
    Ok(())
}
