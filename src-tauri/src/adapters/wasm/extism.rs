use std::collections::HashMap;

use extism::{Plugin, Manifest, Wasm};

use crate::core::error::{CoreError, CommonError};
use super::{PluginMetadata, PluginType};

pub struct ExtismPluginManager {
    plugins: HashMap<String, Plugin>,
    metadata: HashMap<String, PluginMetadata>,
}

impl ExtismPluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn load_plugin(&mut self, id: &str, wasm_bytes: &[u8]) -> Result<PluginMetadata, CoreError> {
        let manifest = Manifest::new([Wasm::data(wasm_bytes.to_vec())]);
        let plugin = Plugin::new(&manifest, [], true)
            .map_err(|e| CoreError::common(CommonError::Internal(format!("Failed to load plugin {}: {}", id, e))))?;
        
        let metadata = PluginMetadata {
            id: id.to_string(),
            name: id.to_string(),
            version: "0.1.0".to_string(),
            plugin_type: PluginType::Tool,
            description: String::new(),
            author: String::new(),
            entry_point: "main".to_string(),
        };

        self.plugins.insert(id.to_string(), plugin);
        self.metadata.insert(id.to_string(), metadata.clone());
        Ok(metadata)
    }

    pub fn unload_plugin(&mut self, id: &str) -> Result<(), CoreError> {
        self.plugins.remove(id);
        self.metadata.remove(id);
        Ok(())
    }

    pub fn list_plugins(&self) -> Vec<PluginMetadata> {
        self.metadata.values().cloned().collect()
    }

    pub fn call_plugin(&mut self, id: &str, func: &str, input: &[u8]) -> Result<Vec<u8>, CoreError> {
        let plugin = self.plugins.get_mut(id)
            .ok_or_else(|| CoreError::common(CommonError::Internal(format!("Plugin {} not found", id))))?;
        plugin.call(func, input)
            .map_err(|e| CoreError::common(CommonError::Internal(format!("Plugin call failed: {}", e))))
    }
}
