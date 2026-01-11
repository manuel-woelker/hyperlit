use crate::engine::HyperlitEngine;
use hyperlit_base::FilePath;
use hyperlit_base::result::HyperlitResult;
use hyperlit_pal::PalHandle;
use hyperlit_pal_real::PalReal;
use std::fs;
use std::io::Read;

pub fn create_html() -> HyperlitResult<()> {
    let pal = PalHandle::new(PalReal::new());
    let engine = HyperlitEngine::new_handle(pal.clone());
    engine.init();
    let output_path = FilePath::from("output");
    pal.remove_directory_all(&output_path)?;
    pal.create_directory_all(&output_path.join_normalized("css"))?;
    let book_html = "engine.render_book_html()?";
    let title = "engine.get_book_title()?";
    let mut index_html = format!(
        r#"
    <!DOCTYPE html>
<html>
<head>
    <title>{title}</title>
    <link rel="stylesheet" href="css/layout.css">
    <link rel="stylesheet" href="css/style.css">
        <link rel="icon"
          type="image/png"
          href="css/favicon.png">
</head>
<body>
    "#
    );
    index_html += book_html;
    index_html += "</body></html>\n";
    let mut index_file = pal.create_file(&output_path.join_normalized("index.html"))?;
    index_file.write_all(index_html.as_bytes())?;

    for filename in ["style.css", "layout.css", "favicon.png"] {
        let mut input_file = fs::File::open(format!("ui_old/css/{filename}"))?;
        let mut bytes = vec![];
        input_file.read_to_end(&mut bytes)?;
        let mut output_file =
            pal.create_file(&output_path.join_normalized(format!("css/{filename}")))?;
        output_file.write_all(&bytes)?;
    }
    Ok(())
}
