use lru::LruCache;
use serde_json::Value;
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};

pub struct Cache {
    cache: LruCache<String, (Value, Instant)>,
    max_age: Duration,
}

impl Cache {
    pub fn new(max_size: usize, max_age: Duration) -> Self {
        Cache {
            cache: LruCache::new(NonZeroUsize::new(max_size).unwrap()),
            max_age,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&Value> {
        if let Some((value, timestamp)) = self.cache.get_mut(key) {
            if timestamp.elapsed() <= self.max_age {
                return Some(value);
            }
        }
        None
    }

    pub fn put(&mut self, key: String, value: Value) {
        let timestamp = Instant::now();
        self.cache.put(key, (value, timestamp));

        if self.cache.len() > self.cache.cap().get() {
            self.cache.pop_lru();
        }
    }

    pub fn remove_expired_entries(&mut self) {
        let now = Instant::now();

        let expired_keys: Vec<String> = self
            .cache
            .iter()
            .filter(|(_, (_, timestamp))| now.duration_since(*timestamp) > self.max_age)
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            self.cache.pop(&key);
        }
    }
}
