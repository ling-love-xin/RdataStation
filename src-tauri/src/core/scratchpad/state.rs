use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::core::scratchpad::ScratchpadStore;

pub struct ScratchpadState {
    pub store: Arc<Mutex<Option<ScratchpadStore>>>,
}

impl ScratchpadState {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(None)),
        }
    }

    pub fn init(&self, project_path: PathBuf) {
        let store = ScratchpadStore::new(project_path);
        tokio::task::block_in_place(|| {
            futures::executor::block_on(async {
                let mut guard = self.store.lock().await;
                *guard = Some(store);
            });
        });
    }
}

impl Default for ScratchpadState {
    fn default() -> Self {
        Self::new()
    }
}
