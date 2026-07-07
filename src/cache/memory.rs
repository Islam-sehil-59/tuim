use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};

use crate::cache::keys::CacheKey;

#[derive(Clone)]
struct CacheEntry<T> {
    value: T,
    inserted_at: Instant,
}

pub struct MemoryCache<T> {
    items: HashMap<CacheKey, CacheEntry<T>>,
    order: VecDeque<CacheKey>,
    ttl: Duration,
    max_items: usize,
}

impl<T: Clone> MemoryCache<T> {
    pub fn new(ttl: Duration, max_items: usize) -> Self {
        Self {
            items: HashMap::new(),
            order: VecDeque::new(),
            ttl,
            max_items,
        }
    }

    pub fn get(&mut self, key: &CacheKey) -> Option<T> {
        let entry = self.items.get(key)?;
        if entry.inserted_at.elapsed() > self.ttl {
            self.items.remove(key);
            self.order.retain(|candidate| candidate != key);
            return None;
        }

        Some(entry.value.clone())
    }

    pub fn insert(&mut self, key: CacheKey, value: T) {
        if !self.items.contains_key(&key) {
            self.order.push_back(key.clone());
        }

        self.items.insert(
            key,
            CacheEntry {
                value,
                inserted_at: Instant::now(),
            },
        );
        self.evict_overflow();
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn evict_overflow(&mut self) {
        while self.items.len() > self.max_items {
            let Some(oldest) = self.order.pop_front() else {
                break;
            };
            self.items.remove(&oldest);
        }
    }
}
