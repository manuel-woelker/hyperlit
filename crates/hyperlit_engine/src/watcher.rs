/* 📖 # Why use a dedicated file watcher thread?

The file watcher runs on a background thread to automatically update documents when
source files change. This eliminates the need to restart Hyperlit when editing documentation.

Key design decisions:

1. **Threading Model**: Uses std::thread (matches the HTTP server pattern)
2. **Store Updates**: Leverages existing thread-safe StoreHandle (Arc<RwLock<dyn DocumentStore>>)
3. **Error Handling**: Logs errors but continues watching - file watching is a convenience feature
4. **Debouncing**: 100ms window to handle editors that save multiple times rapidly
5. **Document Removal**: Scans all documents to find matches by file path

The watcher monitors configured directories recursively and:
- **Create/Modify**: Re-extracts documents and updates the store
- **Delete**: Removes documents from the deleted file
- **Rename**: Handles as delete + create
*/

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use globset::{Glob, GlobSetBuilder};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use tracing::{debug, info, warn};

use hyperlit_base::{FilePath, HyperlitResult, PalHandle, err};

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
/// The watcher runs on a background thread and automatically updates the document store
/// when files change. When dropped, the watcher thread is signaled to stop and joined.
pub struct FileWatcher {
    watcher_handle: Option<JoinHandle<()>>,
    shutdown_tx: Sender<()>,
}

impl FileWatcher {
    /// Start the file watcher with the given configuration.
    ///
    /// This spawns a background thread that watches configured directories
    /// and updates the document store when files change.
    pub fn start(config: FileWatcherConfig) -> HyperlitResult<Self> {
        let (shutdown_tx, shutdown_rx) = channel();

        let watcher_handle = thread::spawn(move || {
            if let Err(e) = run_watcher(config, shutdown_rx) {
                warn!(error = %e, "File watcher thread terminated with error");
            }
        });

        Ok(Self {
            watcher_handle: Some(watcher_handle),
            shutdown_tx,
        })
    }
}

impl Drop for FileWatcher {
    fn drop(&mut self) {
        // Signal shutdown
        let _ = self.shutdown_tx.send(());

        // Join the thread
        if let Some(handle) = self.watcher_handle.take() {
            let _ = handle.join();
        }
    }
}

/// Internal state for the file watcher.
struct WatcherState {
    pal: PalHandle,
    store: StoreHandle,
    debouncer: Debouncer,
    glob_matcher: globset::GlobSet,
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

/// Run the file watcher loop.
fn run_watcher(config: FileWatcherConfig, shutdown_rx: Receiver<()>) -> HyperlitResult<()> {
    // Build glob matcher from config
    let mut glob_builder = GlobSetBuilder::new();
    for dir_config in &config.config.directory {
        for glob_pattern in &dir_config.globs {
            let glob = Glob::new(glob_pattern)
                .map_err(|e| err!("Invalid glob pattern '{}': {}", glob_pattern, e))?;
            glob_builder.add(glob);
        }
    }
    let glob_matcher = glob_builder
        .build()
        .map_err(|e| err!("Failed to build glob matcher: {}", e))?;

    let mut state = WatcherState {
        pal: config.pal.clone(),
        store: config.store.clone(),
        debouncer: Debouncer::new(config.debounce_duration),
        glob_matcher,
    };

    // Create notify watcher
    let (event_tx, event_rx) = channel();
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Err(e) = event_tx.send(res) {
                warn!(error = %e, "Failed to send file watcher event");
            }
        },
        notify::Config::default(),
    )
    .map_err(|e| err!("Failed to create file watcher: {}", e))?;

    // Watch all configured directories
    for dir_config in &config.config.directory {
        for path in &dir_config.paths {
            let watch_path = FilePath::from(path.as_str());
            debug!(path = %watch_path, "Watching directory");

            if let Err(e) = watcher.watch(watch_path.as_path(), RecursiveMode::Recursive) {
                warn!(path = %watch_path, error = %e, "Failed to watch directory");
            }
        }
    }

    // Event loop
    loop {
        // Check for shutdown signal (non-blocking)
        if shutdown_rx.try_recv().is_ok() {
            debug!("File watcher received shutdown signal");
            break;
        }

        // Process events with timeout to allow checking shutdown signal
        match event_rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(event)) => {
                process_event(event, &mut state);
            }
            Ok(Err(e)) => {
                warn!(error = %e, "File watcher error");
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Normal timeout - continue loop to check shutdown
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                warn!("File watcher event channel disconnected");
                break;
            }
        }
    }

    Ok(())
}

/// Process a single file system event.
fn process_event(event: Event, state: &mut WatcherState) {
    // Filter by event kind
    match event.kind {
        EventKind::Create(_) | EventKind::Modify(_) => {
            for path in event.paths {
                let file_path = FilePath::from(path.to_string_lossy().as_ref());

                // Check if file matches glob patterns
                if !state.glob_matcher.is_match(file_path.as_path()) {
                    continue;
                }

                // Apply debouncing
                if !state.debouncer.should_process(&file_path) {
                    continue;
                }

                debug!(file = %file_path, "File modified");
                handle_file_modified(&file_path, &state.pal, &state.store);
            }
        }
        EventKind::Remove(_) => {
            for path in event.paths {
                let file_path = FilePath::from(path.to_string_lossy().as_ref());

                // Check if file matches glob patterns
                if !state.glob_matcher.is_match(file_path.as_path()) {
                    continue;
                }

                debug!(file = %file_path, "File deleted");
                handle_file_deleted(&file_path, &state.store);
            }
        }
        _ => {
            // Ignore other event types (access, metadata changes, etc.)
        }
    }
}

/// Handle a file modification or creation event.
///
/// This function:
/// 1. Removes all existing documents from the file
/// 2. Re-extracts documents from the file
/// 3. Inserts the updated documents into the store
fn handle_file_modified(file_path: &FilePath, pal: &PalHandle, store: &StoreHandle) {
    // First, remove all existing documents from this file
    handle_file_deleted(file_path, store);

    // Extract documents from the modified file
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

/// Handle a file deletion event.
///
/// This function scans all documents in the store and removes those
/// that originated from the deleted file.
fn handle_file_deleted(file_path: &FilePath, store: &StoreHandle) {
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
        info!(file = %file_path, count = removed_count, "Removed documents from deleted file");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
