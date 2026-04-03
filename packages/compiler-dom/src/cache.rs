//! Compilation cache — content-addressed caching of compiled output.
//!
//! Hashes the IR JSON input and stores the compiled HTML. On cache hit,
//! skips compilation entirely.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// A compilation cache backed by `.voce/cache/`.
pub struct CompilationCache {
    cache_dir: PathBuf,
}

impl CompilationCache {
    /// Create a new cache rooted at the given project directory.
    pub fn new(project_dir: &Path) -> Self {
        let cache_dir = project_dir.join(".voce/cache");
        Self { cache_dir }
    }

    /// Look up a cached result for the given IR JSON.
    /// Returns the cached HTML if found and valid.
    pub fn get(&self, ir_json: &str) -> Option<String> {
        let key = cache_key(ir_json);
        let path = self.cache_dir.join(format!("{key}.html"));
        std::fs::read_to_string(path).ok()
    }

    /// Store a compiled result in the cache.
    pub fn put(&self, ir_json: &str, html: &str) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.cache_dir)?;
        let key = cache_key(ir_json);
        let path = self.cache_dir.join(format!("{key}.html"));
        std::fs::write(path, html)?;
        Ok(())
    }

    /// Clear all cached entries.
    pub fn clear(&self) -> std::io::Result<()> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }

    /// Number of cached entries.
    pub fn len(&self) -> usize {
        std::fs::read_dir(&self.cache_dir)
            .map(|entries| entries.count())
            .unwrap_or(0)
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Generate a cache key from IR JSON content.
fn cache_key(ir_json: &str) -> String {
    let mut hasher = DefaultHasher::new();
    ir_json.hash(&mut hasher);
    // Include compiler version in hash so cache invalidates on upgrades
    env!("CARGO_PKG_VERSION").hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_deterministic() {
        let k1 = cache_key("test input");
        let k2 = cache_key("test input");
        assert_eq!(k1, k2);
    }

    #[test]
    fn cache_key_differs_for_different_input() {
        let k1 = cache_key("input a");
        let k2 = cache_key("input b");
        assert_ne!(k1, k2);
    }

    #[test]
    fn cache_roundtrip() {
        let dir = std::env::temp_dir().join("voce-cache-test");
        let _ = std::fs::remove_dir_all(&dir);

        let cache = CompilationCache::new(&dir);
        assert!(cache.get("hello").is_none());

        cache.put("hello", "<html>cached</html>").unwrap();
        assert_eq!(cache.get("hello").unwrap(), "<html>cached</html>");
        assert_eq!(cache.len(), 1);

        cache.clear().unwrap();
        assert!(cache.get("hello").is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
