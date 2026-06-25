use crate::output::LintMessage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub mtime: u64,
    pub size: u64,
    pub messages: Vec<LintMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cache {
    entries: HashMap<PathBuf, CacheEntry>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read cache file: {}", e))?;
        let entries: HashMap<PathBuf, CacheEntry> = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse cache file: {}", e))?;
        Ok(Self { entries })
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(&self.entries)
            .map_err(|e| anyhow::anyhow!("Failed to serialize cache: {}", e))?;
        std::fs::write(path, content)
            .map_err(|e| anyhow::anyhow!("Failed to write cache file: {}", e))?;
        Ok(())
    }

    pub fn get(&self, path: &Path, mtime: u64, size: u64) -> Option<&Vec<LintMessage>> {
        self.entries.get(path).and_then(|entry| {
            if entry.mtime == mtime && entry.size == size {
                Some(&entry.messages)
            } else {
                None
            }
        })
    }

    pub fn insert(&mut self, path: PathBuf, mtime: u64, size: u64, messages: Vec<LintMessage>) {
        self.entries.insert(
            path,
            CacheEntry {
                mtime,
                size,
                messages,
            },
        );
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::{LintMessage, Severity};

    #[test]
    fn test_cache_hit() {
        let mut cache = Cache::new();
        let path = PathBuf::from("test.rs");
        let messages = vec![LintMessage::new(
            1,
            1,
            Severity::Warning,
            "test".to_string(),
            "rule".to_string(),
            None,
        )];
        cache.insert(path.clone(), 123, 456, messages.clone());

        let result = cache.get(&path, 123, 456);
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_cache_miss_mtime_changed() {
        let mut cache = Cache::new();
        let path = PathBuf::from("test.rs");
        cache.insert(path.clone(), 123, 456, vec![]);

        assert!(cache.get(&path, 124, 456).is_none());
    }

    #[test]
    fn test_cache_miss_size_changed() {
        let mut cache = Cache::new();
        let path = PathBuf::from("test.rs");
        cache.insert(path.clone(), 123, 456, vec![]);

        assert!(cache.get(&path, 123, 457).is_none());
    }

    #[test]
    fn test_cache_save_and_load() -> anyhow::Result<()> {
        let temp_file = tempfile::NamedTempFile::new()?;
        let path = PathBuf::from("test.rs");
        let messages = vec![LintMessage::new(
            1,
            1,
            Severity::Warning,
            "test".to_string(),
            "rule".to_string(),
            None,
        )];

        let mut cache = Cache::new();
        cache.insert(path.clone(), 123, 456, messages);
        cache.save(temp_file.path())?;

        let loaded = Cache::load(temp_file.path())?;
        assert!(loaded.get(&path, 123, 456).is_some());
        assert!(loaded.get(&path, 124, 456).is_none());

        Ok(())
    }
}
