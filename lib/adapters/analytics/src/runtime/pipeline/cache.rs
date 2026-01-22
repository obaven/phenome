use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::time::{Duration, Instant};

#[derive(Debug)]
struct CacheEntry<V> {
    value: V,
    expires_at: Instant,
}

#[derive(Debug)]
pub struct TimedLruCache<K, V> {
    entries: HashMap<K, CacheEntry<V>>,
    order: VecDeque<K>,
    ttl: Duration,
    max_entries: usize,
}

impl<K, V> TimedLruCache<K, V>
where
    K: Eq + Hash + Clone,
{
    pub fn new(ttl: Duration, max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            order: VecDeque::new(),
            ttl,
            max_entries,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.evict_expired();
        if self.entries.contains_key(key) {
            self.promote(key);
        }
        self.entries.get(key).map(|entry| &entry.value)
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.evict_expired();
        if self.entries.contains_key(&key) {
            self.promote(&key);
        } else {
            self.order.push_back(key.clone());
        }
        let expires_at = Instant::now() + self.ttl;
        self.entries.insert(key.clone(), CacheEntry { value, expires_at });
        self.evict_overflow();
    }

    fn promote(&mut self, key: &K) {
        if let Some(position) = self.order.iter().position(|item| item == key) {
            self.order.remove(position);
        }
        self.order.push_back(key.clone());
    }

    fn evict_expired(&mut self) {
        let now = Instant::now();
        let expired: Vec<K> = self
            .entries
            .iter()
            .filter(|(_, entry)| entry.expires_at <= now)
            .map(|(key, _)| key.clone())
            .collect();
        for key in expired {
            self.entries.remove(&key);
            if let Some(position) = self.order.iter().position(|item| item == &key) {
                self.order.remove(position);
            }
        }
    }

    fn evict_overflow(&mut self) {
        while self.entries.len() > self.max_entries {
            if let Some(key) = self.order.pop_front() {
                self.entries.remove(&key);
            } else {
                break;
            }
        }
    }
}
