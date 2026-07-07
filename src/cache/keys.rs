use std::fmt;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CacheKey(String);

impl CacheKey {
    pub fn new(namespace: &str, value: impl fmt::Display) -> Self {
        Self(format!("{namespace}:{}", value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
