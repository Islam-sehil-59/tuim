use std::{thread, time::Duration};

use tuim::cache::{keys::CacheKey, memory::MemoryCache};

#[test]
fn cache_key_uses_namespace_prefix() {
    let key = CacheKey::new("search", "nas");

    assert_eq!(key.as_str(), "search:nas");
}

#[test]
fn memory_cache_returns_inserted_values_before_ttl() {
    let mut cache = MemoryCache::new(Duration::from_secs(30), 8);
    let key = CacheKey::new("track", 42);

    cache.insert(key.clone(), "value".to_string());

    assert_eq!(cache.get(&key).as_deref(), Some("value"));
}

#[test]
fn memory_cache_expires_values_after_ttl() {
    let mut cache = MemoryCache::new(Duration::from_millis(1), 8);
    let key = CacheKey::new("track", 42);

    cache.insert(key.clone(), "value".to_string());
    thread::sleep(Duration::from_millis(3));

    assert_eq!(cache.get(&key), None);
}

#[test]
fn memory_cache_evicts_oldest_entry_when_full() {
    let mut cache = MemoryCache::new(Duration::from_secs(30), 1);
    let first = CacheKey::new("track", 1);
    let second = CacheKey::new("track", 2);

    cache.insert(first.clone(), "first".to_string());
    cache.insert(second.clone(), "second".to_string());

    assert_eq!(cache.len(), 1);
    assert_eq!(cache.get(&first), None);
    assert_eq!(cache.get(&second).as_deref(), Some("second"));
}
