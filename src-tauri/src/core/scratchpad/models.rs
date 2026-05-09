use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScratchpadEntryKind {
    File,
    Folder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScratchpadEntry {
    pub name: String,
    pub path: PathBuf,
    pub kind: ScratchpadEntryKind,
    pub size: u64,
    pub modified_at: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchMatch {
    pub file: String,
    pub line_number: usize,
    pub line_content: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchResult {
    pub matches: Vec<SearchMatch>,
    pub total_files_scanned: usize,
    pub total_files_skipped: usize,
    pub skipped_files: Vec<String>,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalReference {
    pub alias: String,
    pub path: PathBuf,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzableFile {
    pub name: String,
    pub relative_path: String,
    pub file_type: String,
    pub size_bytes: u64,
    pub duckdb_query_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileMeta {
    pub last_connection_id: Option<String>,
    pub last_executed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScratchpadConfig {
    pub external_references: Vec<ExternalReference>,
    #[serde(default)]
    pub file_meta: HashMap<String, FileMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScratchpadResponse {
    pub local_entries: Vec<ScratchpadEntry>,
    pub external_references: Vec<ExternalReference>,
    pub scratchpad_path: PathBuf,
    pub file_meta: HashMap<String, FileMeta>,
}
