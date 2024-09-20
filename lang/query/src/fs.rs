#[cfg(not(target_arch = "wasm32"))]
use std::time;
#[cfg(target_arch = "wasm32")]
use web_time as time;

#[cfg(not(target_arch = "wasm32"))]
pub use file_system::FileSystemSource;

use ast::HashMap;
use url::Url;

use crate::result::DriverError;

pub trait FileSource: Send + Sync {
    /// Instruct the source to manage a file with the given URI
    ///
    /// Typically used when keeping the source in-memory
    fn manage(&mut self, uri: &Url) -> bool;
    /// Check if the source manages a file with the given URI
    fn manages(&self, uri: &Url) -> bool;
    /// Read the contents of a file with the given URI
    fn read_to_string(&mut self, uri: &Url) -> Result<String, DriverError>;
    /// Write the contents of a file with the given URI
    ///
    /// Depending on the source, this may write to disk or to memory
    fn write_string(&mut self, uri: &Url, source: &str) -> Result<(), DriverError>;
    /// Check if a file with the given URI has been modified since the last time it was read
    fn is_modified(&self, uri: &Url) -> Result<bool, DriverError>;
    /// Get the last time a file was read
    fn last_retrieved(&self, uri: &Url) -> Option<time::Instant>;
    /// If a URI is requested that is not managed by this source, fall back to another source
    fn fallback_to<S: FileSource>(self, fallback: S) -> OverlaySource<Self, S>
    where
        Self: Sized,
    {
        OverlaySource::new(self, fallback)
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod file_system {
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    use super::*;

    /// A file source that reads from and writes to the file system
    pub struct FileSystemSource {
        root: PathBuf,
        last_retrieved: HashMap<Url, time::SystemTime>,
    }

    impl FileSystemSource {
        pub fn new<P: AsRef<Path>>(root: P) -> Self {
            Self { root: root.as_ref().to_path_buf(), last_retrieved: HashMap::default() }
        }
    }

    impl FileSource for FileSystemSource {
        fn manage(&mut self, uri: &Url) -> bool {
            self.root.join(uri.path()).exists()
        }

        fn manages(&self, uri: &Url) -> bool {
            self.root.join(uri.path()).exists()
        }

        fn read_to_string(&mut self, uri: &Url) -> Result<String, DriverError> {
            let path = self.root.join(uri.path());
            let source =
                std::fs::read_to_string(&path).map_err(Arc::new).map_err(DriverError::Io)?;
            self.last_retrieved.insert(
                uri.clone(),
                std::fs::metadata(&path)
                    .map_err(Arc::new)
                    .map_err(DriverError::Io)?
                    .modified()
                    .map_err(Arc::new)
                    .map_err(DriverError::Io)?,
            );
            Ok(source)
        }

        fn write_string(&mut self, uri: &Url, source: &str) -> Result<(), DriverError> {
            let path = self.root.join(uri.path());
            std::fs::write(&path, source).map_err(Arc::new).map_err(DriverError::Io)?;
            self.last_retrieved.insert(
                uri.clone(),
                std::fs::metadata(&path)
                    .map_err(Arc::new)
                    .map_err(DriverError::Io)?
                    .modified()
                    .map_err(Arc::new)
                    .map_err(DriverError::Io)?,
            );
            Ok(())
        }

        fn last_retrieved(&self, uri: &Url) -> Option<time::Instant> {
            self.last_retrieved.get(uri).and_then(system_time_to_instant)
        }

        fn is_modified(&self, uri: &Url) -> Result<bool, DriverError> {
            let path = self.root.join(uri.path());
            let metadata = std::fs::metadata(&path).map_err(Arc::new).map_err(DriverError::Io)?;
            let time = metadata.modified().map_err(Arc::new).map_err(DriverError::Io)?;
            Ok(self.last_retrieved.get(uri).map_or(true, |last| time > *last))
        }
    }

    fn system_time_to_instant(system_time: &time::SystemTime) -> Option<time::Instant> {
        let now = time::Instant::now();
        let system_now = time::SystemTime::now();

        match system_time.duration_since(system_now) {
            Ok(duration) => Some(now + duration),
            Err(_) => None,
        }
    }
}

/// A file source that keeps files in memory
pub struct InMemorySource {
    files: HashMap<Url, String>,
    modified: HashMap<Url, bool>,
}

impl Default for InMemorySource {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemorySource {
    pub fn new() -> Self {
        Self { files: HashMap::default(), modified: HashMap::default() }
    }

    pub fn insert(&mut self, uri: Url, source: String) {
        self.files.insert(uri.clone(), source);
        self.modified.insert(uri, true);
    }
}

impl FileSource for InMemorySource {
    fn manage(&mut self, uri: &Url) -> bool {
        self.files.insert(uri.clone(), String::default());
        self.modified.insert(uri.clone(), false);
        true
    }

    fn manages(&self, uri: &Url) -> bool {
        self.files.contains_key(uri)
    }

    fn read_to_string(&mut self, uri: &Url) -> Result<String, DriverError> {
        if self.manages(uri) {
            self.modified.insert(uri.clone(), false);
            Ok(self.files.get(uri).cloned().unwrap_or_default())
        } else {
            Err(DriverError::FileNotFound(uri.to_owned()))
        }
    }

    fn write_string(&mut self, uri: &Url, source: &str) -> Result<(), DriverError> {
        self.files.insert(uri.clone(), source.to_string());
        self.modified.insert(uri.clone(), true);
        Ok(())
    }

    fn last_retrieved(&self, uri: &Url) -> Option<time::Instant> {
        self.modified.get(uri).map(|_| time::Instant::now())
    }

    fn is_modified(&self, uri: &Url) -> Result<bool, DriverError> {
        Ok(*self.modified.get(uri).unwrap_or(&true))
    }
}

/// A source that first tries to access files from the first source, and falls back to the second
pub struct OverlaySource<S1, S2> {
    first: S1,
    second: S2,
}

impl<S1, S2> OverlaySource<S1, S2> {
    pub fn new(first: S1, second: S2) -> Self {
        Self { first, second }
    }
}

impl<S1, S2> FileSource for OverlaySource<S1, S2>
where
    S1: FileSource,
    S2: FileSource,
{
    fn manage(&mut self, uri: &Url) -> bool {
        self.first.manage(uri) || self.second.manage(uri)
    }

    fn manages(&self, uri: &Url) -> bool {
        self.first.manages(uri) || self.second.manages(uri)
    }

    fn read_to_string(&mut self, uri: &Url) -> Result<String, DriverError> {
        match self.first.read_to_string(uri) {
            Ok(source) => Ok(source),
            Err(DriverError::FileNotFound(_)) => self.second.read_to_string(uri),
            Err(err) => Err(err),
        }
    }

    fn write_string(&mut self, uri: &Url, source: &str) -> Result<(), DriverError> {
        if self.first.manages(uri) {
            self.first.write_string(uri, source)
        } else {
            self.second.write_string(uri, source)
        }
    }

    fn last_retrieved(&self, uri: &Url) -> Option<time::Instant> {
        if self.first.manages(uri) {
            self.first.last_retrieved(uri)
        } else {
            self.second.last_retrieved(uri)
        }
    }

    fn is_modified(&self, uri: &Url) -> Result<bool, DriverError> {
        if self.first.manages(uri) {
            self.first.is_modified(uri)
        } else {
            self.second.is_modified(uri)
        }
    }
}
