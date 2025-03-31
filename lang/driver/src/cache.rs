use url::Url;

use ast::HashMap;

pub struct Cache<T> {
    entries: HashMap<Url, CacheEntry<T>>,
}

impl<T> Default for Cache<T> {
    fn default() -> Self {
        Self { entries: HashMap::default() }
    }
}

impl<T> Cache<T> {
    pub fn keys(&self) -> impl Iterator<Item = &Url> {
        self.entries.keys()
    }

    /// Get the value associated with a URI regardless of staleness
    pub fn get_even_if_stale(&self, uri: &Url) -> Option<&T> {
        self.entries.get(uri).map(|entry| entry.value())
    }

    /// Get the value associated with a URI if it is not stale
    pub fn get_unless_stale(&self, uri: &Url) -> Option<&T> {
        if self.is_stale(uri) { None } else { self.get_even_if_stale(uri) }
    }

    pub fn insert(&mut self, uri: Url, value: T) {
        self.entries.insert(uri, CacheEntry::from(value));
    }

    pub fn is_stale(&self, uri: &Url) -> bool {
        self.entries.get(uri).map(|entry| entry.stale).unwrap_or(true)
    }

    pub fn invalidate(&mut self, uri: &Url) {
        self.entries.entry(uri.clone()).and_modify(|entry| entry.stale = true);
    }
}

pub struct CacheEntry<T> {
    value: T,
    stale: bool,
}

impl<T> CacheEntry<T> {
    pub fn value(&self) -> &T {
        &self.value
    }
}

impl<T> From<T> for CacheEntry<T> {
    fn from(value: T) -> Self {
        Self { value, stale: false }
    }
}
