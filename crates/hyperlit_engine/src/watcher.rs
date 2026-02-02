/* 📖 # Why integrate file watching through the PAL?

The file watcher coordinates automatic document updates when source files change,
but delegates the actual filesystem watching to the PAL (Platform Abstraction Layer).

This design:
1. **Respects the architecture**: All filesystem operations go through the PAL
2. **Enables testing**: MockPal can simulate file changes for testing
3. **Centralizes platform code**: notify crate usage is isolated in RealPal
4. **Simplifies the engine**: Engine focuses on document extraction, PAL handles watching

The watcher creates a callback that extracts documents and updates the store,
then registers this callback with PAL.watch_directory() for each configured directory.
*/

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tracing::{debug, info, warn};

use hyperlit_base::pal::FileChangeEvent;
use hyperlit_base::{FilePath, HyperlitResult, PalHandle};

use crate::{Config, StoreHandle, extract_documents};

/// Configuration for the file watcher.
#[derive(Clone)]
pub struct FileWatcherConfig {
    config: Config,
    pal: PalHandle,
    store: StoreHandle,
    debounce_duration: Duration,
}

impl FileWatcherConfig {
    /// Create a new file watcher configuration.
    pub fn new(
        config: Config,
        pal: PalHandle,
        store: StoreHandle,
        debounce_duration: Duration,
    ) -> Self {
        Self {
            config,
            pal,
            store,
            debounce_duration,
        }
    }
}

/// Handle to a running file watcher.
///
/// The watcher monitors configured directories and automatically updates the document store
/// when files change. The actual watching is handled by the PAL.
pub struct FileWatcher;

impl FileWatcher {
    /// Start the file watcher with the given configuration.
    ///
    /// This registers watch callbacks with the PAL for each configured directory.
    /// The PAL handles the actual file system monitoring.
    pub fn start(config: FileWatcherConfig) -> HyperlitResult<Self> {
        // Create shared debouncer wrapped in Arc<Mutex<>> for thread-safe access
        let debouncer = Arc::new(Mutex::new(Debouncer::new(config.debounce_duration)));

        // Register watchers for each directory
        for dir_config in &config.config.directory {
            for path in &dir_config.paths {
                let file_path = FilePath::from(path.as_str());
                let pal_clone = config.pal.clone();
                let store_clone = config.store.clone();
                let debouncer_clone = debouncer.clone();

                // Create callback for this directory
                let callback = Box::new(move |event: FileChangeEvent| {
                    for changed_file in event.changed_files {
                        // Apply debouncing
                        {
                            let mut debouncer = debouncer_clone.lock().unwrap();
                            if !debouncer.should_process(&changed_file) {
                                continue;
                            }
                        }

                        debug!(file = %changed_file, "File changed");
                        handle_file_change(&changed_file, &pal_clone, &store_clone);
                    }
                });

                // Register the watcher with the PAL
                config
                    .pal
                    .watch_directory(&file_path, &dir_config.globs, callback)?;
            }
        }

        Ok(Self)
    }
}

/// Debouncer to handle rapid file change events.
///
/// Editors often save files multiple times in quick succession. The debouncer
/// filters out duplicate events within a time window.
struct Debouncer {
    last_events: HashMap<FilePath, Instant>,
    debounce_duration: Duration,
}

impl Debouncer {
    fn new(debounce_duration: Duration) -> Self {
        Self {
            last_events: HashMap::new(),
            debounce_duration,
        }
    }

    /// Check if an event should be processed, updating the last event time.
    ///
    /// Returns true if enough time has passed since the last event for this file.
    fn should_process(&mut self, file_path: &FilePath) -> bool {
        let now = Instant::now();

        if let Some(last_time) = self.last_events.get(file_path)
            && now.duration_since(*last_time) < self.debounce_duration
        {
            debug!(file = %file_path, "Debouncing file change event");
            return false;
        }

        self.last_events.insert(file_path.clone(), now);
        true
    }
}

/// Handle a file change event (creation, modification, or deletion).
///
/// This function:
/// 1. Removes all existing documents from the file
/// 2. Re-extracts documents from the file (if it still exists)
/// 3. Inserts the updated documents into the store
fn handle_file_change(file_path: &FilePath, pal: &PalHandle, store: &StoreHandle) {
    // First, remove all existing documents from this file
    remove_documents_for_file(file_path, store);

    // Check if file still exists - if not, we're done (it was deleted)
    match pal.file_exists(file_path) {
        Ok(true) => {
            // File exists - extract and insert documents
            match extract_documents(pal, std::slice::from_ref(file_path)) {
                Ok(extraction) => {
                    let mut inserted_count = 0;

                    for doc in extraction.documents {
                        match store.insert(doc) {
                            Ok(_) => inserted_count += 1,
                            Err(e) => {
                                warn!(file = %file_path, error = %e, "Failed to insert document into store");
                            }
                        }
                    }

                    if !extraction.errors.is_empty() {
                        for error in extraction.errors {
                            warn!(file = %error.file_path, error = ?error.error, "Failed to extract documents from modified file");
                        }
                    }

                    if inserted_count > 0 {
                        info!(file = %file_path, count = inserted_count, "Updated documents from modified file");
                    }
                }
                Err(e) => {
                    warn!(file = %file_path, error = %e, "Failed to extract documents from modified file");
                }
            }
        }
        Ok(false) => {
            // File was deleted - already removed documents above
            debug!(file = %file_path, "File deleted");
        }
        Err(e) => {
            warn!(file = %file_path, error = %e, "Failed to check if file exists");
        }
    }
}

/// Remove all documents that originated from the given file.
///
/// This function scans all documents in the store and removes those
/// that match the file path.
fn remove_documents_for_file(file_path: &FilePath, store: &StoreHandle) {
    // Find all documents from this file
    let documents_to_remove = match store.list() {
        Ok(docs) => docs
            .into_iter()
            .filter(|doc| doc.source().file_path() == file_path)
            .map(|doc| doc.id().clone())
            .collect::<Vec<_>>(),
        Err(e) => {
            warn!(file = %file_path, error = %e, "Failed to list documents for deletion");
            return;
        }
    };

    let mut removed_count = 0;
    for doc_id in documents_to_remove {
        match store.remove(&doc_id) {
            Ok(Some(_)) => removed_count += 1,
            Ok(None) => {
                debug!(doc_id = %doc_id, "Document already removed");
            }
            Err(e) => {
                warn!(doc_id = %doc_id, error = %e, "Failed to remove document");
            }
        }
    }

    if removed_count > 0 {
        info!(file = %file_path, count = removed_count, "Removed documents from file");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_debouncer() {
        let mut debouncer = Debouncer::new(Duration::from_millis(100));
        let file_path = FilePath::from("test.rs");

        // First event should be processed
        assert!(debouncer.should_process(&file_path));

        // Immediate second event should be debounced
        assert!(!debouncer.should_process(&file_path));

        // After waiting, event should be processed
        thread::sleep(Duration::from_millis(150));
        assert!(debouncer.should_process(&file_path));
    }
}
