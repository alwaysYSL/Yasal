use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

pub struct IconCache {
    cache_dir: PathBuf,
}

impl IconCache {
    pub fn new() -> Self {
        let local_data = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        let cache_dir = local_data.join("Yasal").join("IconCache");
        let _ = fs::create_dir_all(&cache_dir);
        Self { cache_dir }
    }

    /// Computes deterministic filename for an application or file path.
    pub fn get_icon_path(&self, identifier: &str) -> PathBuf {
        let mut hasher = DefaultHasher::new();
        identifier.hash(&mut hasher);
        let hash = hasher.finish();
        self.cache_dir.join(format!("{:016x}.png", hash))
    }

    /// Checks if the icon is already extracted in cache.
    pub fn is_cached(&self, identifier: &str) -> bool {
        self.get_icon_path(identifier).exists()
    }
}

impl Default for IconCache {
    fn default() -> Self {
        Self::new()
    }
}
