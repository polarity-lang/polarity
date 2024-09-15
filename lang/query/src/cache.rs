#[cfg(not(target_arch = "wasm32"))]
use std::time;
#[cfg(target_arch = "wasm32")]
use web_time as time;

use url::Url;

use ast::HashMap;

use crate::Database;

type IsStaleFn = dyn Fn(&Database, &Url, time::Instant) -> bool + Send + Sync;

pub struct Cache<T> {
    entries: HashMap<Url, CacheEntry<T>>,
    is_stale: Box<IsStaleFn>,
}

impl<T> Cache<T> {
    pub fn new(
        is_stale: impl Fn(&Database, &Url, time::Instant) -> bool + 'static + Send + Sync,
    ) -> Self {
        Self { entries: HashMap::default(), is_stale: Box::new(is_stale) }
    }

    /// Get the value associated with a URI regardless of staleness
    pub fn get_even_if_stale(&self, uri: &Url) -> Option<&T> {
        self.entries.get(uri).map(|entry| entry.value())
    }

    /// Get the value associated with a URI if it is not stale
    pub fn get_unless_stale(&self, db: &Database, uri: &Url) -> Option<&T> {
        if self.is_stale(db, uri) {
            None
        } else {
            self.get_even_if_stale(uri)
        }
    }

    pub fn insert(&mut self, uri: Url, value: T) {
        self.entries.insert(uri, CacheEntry::from(value));
    }

    pub fn remove(&mut self, uri: &Url) {
        self.entries.remove(uri);
    }

    pub fn is_stale(&self, db: &Database, uri: &Url) -> bool {
        let Some(entry) = self.entries.get(uri) else {
            return true;
        };
        (self.is_stale)(db, uri, entry.last_modified)
    }

    pub fn last_modified(&self, uri: &Url) -> Option<time::Instant> {
        self.entries.get(uri).map(|entry| entry.last_modified)
    }
}

pub struct CacheEntry<T> {
    value: T,
    last_modified: time::Instant,
}

impl<T> CacheEntry<T> {
    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn last_modified(&self) -> time::Instant {
        self.last_modified
    }
}

impl<T> From<T> for CacheEntry<T> {
    fn from(value: T) -> Self {
        Self { value, last_modified: time::Instant::now() }
    }
}
