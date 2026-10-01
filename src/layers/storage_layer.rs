use std::{
    collections::HashMap,
    fs, io,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

const STORAGE_FILE: &str = "storage.cache";

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    pub value: String,
    pub expired_at: Option<u64>,
}

impl Data {
    fn new(value: String, expired_at: Option<u64>) -> Self {
        Self { value, expired_at }
    }

    fn is_expired(&self) -> bool {
        if let Some(exp) = self.expired_at {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            return now >= exp;
        } else {
            return false;
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Storage {
    pub store: HashMap<String, Data>,
    #[serde(skip)]
    pub cache_hits: u64,
    #[serde(skip)]
    pub cache_misses: u64,
}

impl Storage {
    pub fn new() -> Self {
        Storage {
            store: HashMap::new(),
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    pub fn load_from_disk() -> io::Result<Self> {
        if !Path::new(STORAGE_FILE).exists() {
            return Ok(Self::new());
        }

        let compressed_data = fs::read(STORAGE_FILE)?;

        let json_data = zstd::decode_all(compressed_data.as_slice())?;

        let mut storage: Storage = serde_json::from_slice(&json_data)?;

        storage.cache_hits = 0;
        storage.cache_misses = 0;

        storage.store.retain(|_, data| !data.is_expired());

        Ok(storage)
    }

    pub fn save_to_disk(&self) -> io::Result<()> {
        let json_data = serde_json::to_vec(self)?;

        let compressed_data = zstd::encode_all(json_data.as_slice(), 3)?;

        let temp_file = format!("{STORAGE_FILE}.tmp");

        fs::write(&temp_file, compressed_data)?;

        fs::rename(temp_file, STORAGE_FILE)?;

        Ok(())
    }

    pub fn get(&mut self, key: &str) -> Option<&Data> {
        let expired = self
            .store
            .get(key)
            .map(|data| data.is_expired())
            .unwrap_or(false);

        if expired {
            self.store.remove(key);
            self.cache_misses += 1;

            let _ = self.save_to_disk();

            return None;
        }

        match self.store.get(key) {
            Some(data) => {
                self.cache_hits += 1;
                return Some(data);
            }
            None => {
                self.cache_misses += 1;

                return None;
            }
        }
    }

    pub fn insert(&mut self, key: String, value: String, ttl: Option<u64>) -> io::Result<()> {
        let expiration_time = ttl.map(|exp| {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                + exp;

            now
        });

        self.store.insert(key, Data::new(value, expiration_time));

        self.save_to_disk()?;

        Ok(())
    }

    pub fn delete(&mut self, key: &str) -> io::Result<bool> {
        let is_deleted = self.store.remove(key).is_some();

        if is_deleted {
            self.save_to_disk()?;
        }

        Ok(is_deleted)
    }

    pub fn sweep_expired_cache(&mut self) -> io::Result<()> {
        let expired_keys: Vec<String> = self
            .store
            .iter()
            .filter_map(|(key, value)| {
                if value.is_expired() {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect();

        if expired_keys.is_empty() {
            return Ok(());
        }

        for key in expired_keys {
            self.store.remove(&key);
        }

        self.save_to_disk()
    }
}
