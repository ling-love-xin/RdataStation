use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use include_dir::Dir;
use crate::core::error::{CommonError, CoreError};

use super::rule_types::RuleFile;

pub struct RuleRegistry {
    rules: HashMap<String, RuleFile>,
    by_category: HashMap<String, Vec<String>>,
}

impl RuleRegistry {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            by_category: HashMap::new(),
        }
    }

    pub fn load_from_dir(&mut self, dir: &Path) -> Result<usize, CoreError> {
        if !dir.exists() || !dir.is_dir() {
            return Ok(0);
        }
        let mut count = 0;
        self.scan_directory(dir, &mut count)?;
        Ok(count)
    }

    fn scan_directory(&mut self, dir: &Path, count: &mut usize) -> Result<(), CoreError> {
        let entries = fs::read_dir(dir).map_err(|e| {
            CoreError::common(CommonError::General(format!(
                "Cannot read rules directory '{}': {}",
                dir.display(),
                e
            )))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                CoreError::common(CommonError::General(format!("Dir entry error: {}", e)))
            })?;
            let path = entry.path();

            if path.is_dir() {
                self.scan_directory(&path, count)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Ok(rule) = self.parse_rule_file(&path) {
                    let category = rule.meta.category.clone();
                    let id = rule.meta.id.clone();

                    self.by_category
                        .entry(category)
                        .or_default()
                        .push(id.clone());

                    self.rules.insert(id, rule);
                    *count += 1;
                }
            }
        }
        Ok(())
    }

    pub fn load_from_embedded_dir(&mut self, dir: &Dir) -> Result<usize, CoreError> {
        let mut count = 0;
        for entry in dir.entries() {
            match entry {
                include_dir::DirEntry::Dir(subdir) => {
                    count += self.load_from_embedded_dir(subdir)?;
                }
                include_dir::DirEntry::File(file) => {
                    if file.path().extension().and_then(|s| s.to_str()) == Some("toml") {
                        if let Some(content) = file.contents_utf8() {
                            if let Ok(rule) = self.parse_toml(content) {
                                let category = rule.meta.category.clone();
                                let id = rule.meta.id.clone();
                                self.by_category
                                    .entry(category)
                                    .or_default()
                                    .push(id.clone());
                                self.rules.insert(id, rule);
                                count += 1;
                            }
                        }
                    }
                }
            }
        }
        Ok(count)
    }

    fn parse_rule_file(&self, path: &Path) -> Result<RuleFile, CoreError> {
        let content = fs::read_to_string(path).map_err(|e| {
            CoreError::common(CommonError::General(format!(
                "Cannot read rule file '{}': {}",
                path.display(),
                e
            )))
        })?;
        self.parse_toml(&content)
    }

    fn parse_toml(&self, content: &str) -> Result<RuleFile, CoreError> {
        toml::from_str::<RuleFile>(content).map_err(|e| {
            CoreError::common(CommonError::General(format!("TOML parse error: {}", e)))
        })
    }

    pub fn get(&self, id: &str) -> Option<&RuleFile> {
        self.rules.get(id)
    }

    pub fn list_by_category(&self, category: &str) -> Vec<&RuleFile> {
        self.by_category
            .get(category)
            .map(|ids| ids.iter().filter_map(|id| self.rules.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn rules_for_column_type(&self, column_type: &str) -> Vec<&RuleFile> {
        self.rules
            .values()
            .filter(|r| {
                r.meta
                    .applies_to
                    .iter()
                    .any(|t| t.eq_ignore_ascii_case(column_type))
            })
            .collect()
    }

    pub fn all_rules(&self) -> Vec<&RuleFile> {
        self.rules.values().collect()
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

pub fn get_project_rules_dir(project_path: &Path) -> PathBuf {
    project_path.join(".RSMETA").join("insight-rules")
}
