/* 📖 # Why is the CLI minimal and hardcoded?

The CLI is intentionally kept minimal with no argument parsing or configuration
options. This approach:

1. **Reduces complexity**: No clap or similar dependency needed
2. **Simplifies testing**: Just run `hyperlit` in a directory with hyperlit.toml
3. **Clear conventions**: Always looks for `hyperlit.toml` in current directory
4. **Fast iteration**: Can add arguments later when use cases emerge

The workflow is straightforward:
1. Change to your project directory
2. Ensure `hyperlit.toml` exists
3. Run `hyperlit`
4. Documents are extracted and stored

Exit codes:
- 0: Success (documents extracted and stored)
- 1: Error (config not found, parsing failed, or no documents stored)
*/

use std::env;
use std::process;

use hyperlit_base::tracing::init_tracing;
use hyperlit_base::{FilePath, PalHandle, RealPal};
use hyperlit_engine::store::{InMemoryStore, StoreHandle};
use hyperlit_engine::{extract_documents, load_config, scan_files};

fn main() {
    init_tracing().unwrap();

    let current_dir = env::current_dir().unwrap_or_else(|e| {
        eprintln!("Error: Failed to get current directory: {}", e);
        process::exit(1);
    });

    let pal = PalHandle::new(RealPal::new(current_dir.clone()));

    let config_path = FilePath::from("hyperlit.toml");
    let config = match load_config(&pal, &config_path) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error: Failed to load config from hyperlit.toml: {}", e);
            process::exit(1);
        }
    };

    println!("Configuration loaded: {}", config.title);

    let scan_result = match scan_files(&pal, &config) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error: Failed to scan files: {}", e);
            process::exit(1);
        }
    };

    if !scan_result.errors.is_empty() {
        eprintln!("\nWarnings during file scanning:");
        for error in &scan_result.errors {
            eprintln!("  - {}: {:?}", error.directory_path, error.error);
        }
    }

    println!("Found {} files", scan_result.files.len());

    if scan_result.files.is_empty() {
        println!("No files found matching the configured patterns.");
        process::exit(0);
    }

    let extraction = match extract_documents(&pal, &scan_result.files) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error: Failed to extract documents: {}", e);
            process::exit(1);
        }
    };

    if !extraction.errors.is_empty() {
        eprintln!("\nWarnings during document extraction:");
        for error in &extraction.errors {
            eprintln!("  - {}: {:?}", error.file_path, error.error);
        }
    }

    println!("Extracted {} documents", extraction.documents.len());

    let store = StoreHandle::new(InMemoryStore::with_capacity(extraction.documents.len()));

    let mut success_count = 0;
    for doc in extraction.documents {
        let id = doc.id().clone();
        match store.insert(doc) {
            Ok(_) => {
                success_count += 1;
                println!("  + {}: {}", id, store.get(&id).unwrap().unwrap().title());
            }
            Err(e) => {
                eprintln!("  - Failed to store document {}: {}", id, e);
            }
        }
    }

    println!(
        "\nSuccessfully stored {}/{} documents",
        success_count,
        store.len().unwrap()
    );

    if success_count > 0 {
        process::exit(0);
    } else {
        eprintln!("No documents were successfully stored.");
        process::exit(1);
    }
}
