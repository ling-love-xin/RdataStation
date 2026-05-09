use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::sync::Mutex;

use crate::core::scratchpad::ScratchpadStore;

pub struct ScratchpadState {
    pub store: Arc<Mutex<Option<ScratchpadStore>>>,
    pub watcher_active: Arc<AtomicBool>,
}

impl ScratchpadState {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(None)),
            watcher_active: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn init(&self, project_path: PathBuf) {
        let store = ScratchpadStore::new(project_path);
        let handle = Handle::current();
        handle.block_on(async {
            let mut guard = self.store.lock().await;
            *guard = Some(store);
        });
    }

    pub fn is_watching(&self) -> bool {
        self.watcher_active.load(Ordering::Relaxed)
    }

    pub fn set_watching(&self, active: bool) {
        self.watcher_active.store(active, Ordering::Relaxed);
    }
}

impl Drop for ScratchpadState {
    fn drop(&mut self) {
        self.set_watching(false);
    }
}

impl Default for ScratchpadState {
    fn default() -> Self {
        Self::new()
    }
}
