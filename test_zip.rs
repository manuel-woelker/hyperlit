use std::env;
use std::fs::File;
use std::io::{Cursor, Read};

fn main() {
    // Read current executable
    let exe_path = env::current_exe().unwrap();
    println!("Executable path: {:?}", exe_path);

    let mut file = File::open(&exe_path).unwrap();
    let mut content = Vec::new();
    file.read_to_end(&mut content).unwrap();
    println!("Binary size: {} bytes", content.len());

    // Try to open as zip
    let cursor = Cursor::new(&content);
    match zip::ZipArchive::new(cursor) {
        Ok(mut archive) => {
            println!("✓ Successfully opened zip archive");
            println!("  Files in archive: {}", archive.len());

            // List files
            for i in 0..archive.len() {
                let file = archive.by_index(i).unwrap();
                println!("  - {} ({} bytes)", file.name(), file.size());
            }

            // Try to read index.html
            match archive.by_name("index.html") {
                Ok(mut file) => {
                    let mut contents = Vec::new();
                    file.read_to_end(&mut contents).unwrap();
                    println!("✓ Successfully read index.html ({} bytes)", contents.len());
                }
                Err(e) => {
                    println!("✗ Failed to read index.html: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to open zip archive: {}", e);
        }
    }
}
