use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub value: T,
    pub created_at: u64,
    pub expires_at: Option<u64>,
}

pub struct CacheManager {
    memory: HashMap<String, CacheEntry<Vec<u8>>>,
    disk_path: Option<PathBuf>,
    default_ttl: Duration,
}

impl CacheManager {
    pub fn new() -> Self {
        Self {
            memory: HashMap::new(),
            disk_path: None,
            default_ttl: Duration::from_secs(300),
        }
    }

    pub fn with_disk(mut self, path: PathBuf) -> Self {
        self.disk_path = Some(path);
        self
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = ttl;
        self
    }

    pub fn set(&mut self, key: String, value: Vec<u8>, ttl: Option<Duration>) {
        let now = Instant::now();
        let expires_at = ttl.or(Some(self.default_ttl)).map(|t| {
            now.checked_add(t)
                .map(|i| i.elapsed().as_secs() + now.elapsed().as_secs())
                .unwrap_or(0)
        });

        self.memory.insert(
            key,
            CacheEntry {
                value,
                created_at: now.elapsed().as_secs(),
                expires_at,
            },
        );
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let entry = self.memory.get(key)?;

        if let Some(expires_at) = entry.expires_at {
            let now = Instant::now().elapsed().as_secs();
            if now > expires_at {
                return None;
            }
        }

        Some(entry.value.clone())
    }

    pub fn contains(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    pub fn clear(&mut self) {
        self.memory.clear();
    }

    pub fn remove(&mut self, key: &str) {
        self.memory.remove(key);
    }

    pub fn save_to_disk(&self) -> std::io::Result<()> {
        if let Some(ref path) = self.disk_path {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let serialized = serde_json::to_vec(&self.memory)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            fs::write(path, serialized)?;
        }
        Ok(())
    }

    pub fn load_from_disk(&mut self) -> std::io::Result<()> {
        if let Some(ref path) = self.disk_path {
            if path.exists() {
                let data = fs::read(path)?;
                let loaded: HashMap<String, CacheEntry<Vec<u8>>> = serde_json::from_slice(&data)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                self.memory = loaded;
            }
        }
        Ok(())
    }

    pub fn is_expired(&self, key: &str) -> bool {
        if let Some(entry) = self.memory.get(key) {
            if let Some(expires_at) = entry.expires_at {
                return Instant::now().elapsed().as_secs() > expires_at;
            }
        }
        false
    }

    pub fn cleanup_expired(&mut self) {
        let now = Instant::now().elapsed().as_secs();
        self.memory.retain(|_, entry| {
            if let Some(expires_at) = entry.expires_at {
                now < expires_at
            } else {
                true
            }
        });
    }

    pub fn len(&self) -> usize {
        self.memory.len()
    }

    pub fn is_empty(&self) -> bool {
        self.memory.is_empty()
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_set_get() {
        let mut cache = CacheManager::new();
        cache.set("key1".to_string(), b"value1".to_vec(), None);

        assert_eq!(cache.get("key1"), Some(b"value1".to_vec()));
        assert_eq!(cache.get("nonexistent"), None);
    }

    #[test]
    fn test_cache_expiry() {
        let mut cache = CacheManager::new();
        cache.set(
            "temp".to_string(),
            b"data".to_vec(),
            Some(Duration::from_secs(1)),
        );

        assert!(!cache.is_expired("temp"));
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = CacheManager::new();
        cache.set("key1".to_string(), b"value1".to_vec(), None);
        cache.set("key2".to_string(), b"value2".to_vec(), None);

        assert_eq!(cache.len(), 2);

        cache.clear();

        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_remove() {
        let mut cache = CacheManager::new();
        cache.set("key1".to_string(), b"value1".to_vec(), None);

        assert!(cache.contains("key1"));

        cache.remove("key1");

        assert!(!cache.contains("key1"));
    }
}
