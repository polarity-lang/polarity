use async_trait::async_trait;

#[cfg(not(target_arch = "wasm32"))]
pub use file_system::FileSystemSource;

use polarity_lang_ast::HashMap;
use url::Url;

use crate::result::DriverError;

#[async_trait]
pub trait FileSource: Send + Sync {
    /// Check if a file with the given URI exists
    async fn exists(&mut self, uri: &Url) -> Result<bool, DriverError>;
    /// Instruct the source to register a file with the given URI.
    ///
    /// Typically used when keeping the source in-memory.
    fn register(&mut self, uri: &Url) -> bool;
    /// Stop keeping track of a file with the given URI
    ///
    /// This is mostly relevant for in-memory sources.
    /// Returns `true` if the source had the file registered.
    fn forget(&mut self, uri: &Url) -> bool;
    /// Read the contents of a file with the given URI
    async fn read_to_string(&mut self, uri: &Url) -> Result<String, DriverError>;
    /// Write the contents of a file with the given URI
    ///
    /// Depending on the source, this may write to disk or to memory
    async fn write_string(&mut self, uri: &Url, source: &str) -> Result<(), DriverError>;
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
    }

    impl FileSystemSource {
        pub fn new<P: AsRef<Path>>(root: P) -> Self {
            Self { root: root.as_ref().to_path_buf() }
        }
    }

    #[async_trait]
    impl FileSource for FileSystemSource {
        async fn exists(&mut self, uri: &Url) -> Result<bool, DriverError> {
            let filepath = uri.to_file_path().expect("Failed to convert URI to filepath");
            Ok(self.root.join(filepath).exists())
        }

        fn register(&mut self, uri: &Url) -> bool {
            let filepath = uri.to_file_path().expect("Failed to convert URI to filepath");
            self.root.join(filepath).exists()
        }

        fn forget(&mut self, uri: &Url) -> bool {
            let filepath = uri.to_file_path().expect("Failed to convert URI to filepath");
            self.root.join(filepath).exists()
        }

        async fn read_to_string(&mut self, uri: &Url) -> Result<String, DriverError> {
            let filepath = uri.to_file_path().expect("Failed to convert URI to filepath");
            let path = self.root.join(filepath);
            let source =
                std::fs::read_to_string(&path).map_err(Arc::new).map_err(DriverError::Io)?;
            // Depending on how git is configured on Windows, it may check-out Unix line endings (\n) as Windows line endings (\r\n).
            // To have identical source code spans on all platforms, we replace these by Unix line endings (\n).
            let source = source.replace("\r\n", "\n");
            Ok(source)
        }

        async fn write_string(&mut self, uri: &Url, source: &str) -> Result<(), DriverError> {
            let filepath = uri.to_file_path().expect("Failed to convert URI to filepath");
            let path = self.root.join(filepath);
            std::fs::write(&path, source).map_err(Arc::new).map_err(DriverError::Io)?;
            Ok(())
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

#[async_trait]
impl FileSource for InMemorySource {
    async fn exists(&mut self, uri: &Url) -> Result<bool, DriverError> {
        Ok(self.files.contains_key(uri))
    }

    fn register(&mut self, uri: &Url) -> bool {
        self.files.insert(uri.clone(), String::default());
        self.modified.insert(uri.clone(), false);
        true
    }

    fn forget(&mut self, uri: &Url) -> bool {
        let managed = self.files.remove(uri).is_some();
        self.modified.remove(uri);
        managed
    }

    async fn read_to_string(&mut self, uri: &Url) -> Result<String, DriverError> {
        if self.exists(uri).await? {
            self.modified.insert(uri.clone(), false);
            Ok(self.files.get(uri).cloned().unwrap_or_default())
        } else {
            Err(DriverError::FileNotFound(uri.to_owned()))
        }
    }

    async fn write_string(&mut self, uri: &Url, source: &str) -> Result<(), DriverError> {
        self.files.insert(uri.clone(), source.to_string());
        self.modified.insert(uri.clone(), true);
        Ok(())
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

#[async_trait]
impl<S1, S2> FileSource for OverlaySource<S1, S2>
where
    S1: FileSource,
    S2: FileSource,
{
    async fn exists(&mut self, uri: &Url) -> Result<bool, DriverError> {
        Ok(self.first.exists(uri).await? || self.second.exists(uri).await?)
    }

    fn register(&mut self, uri: &Url) -> bool {
        self.first.register(uri) || self.second.register(uri)
    }

    fn forget(&mut self, uri: &Url) -> bool {
        self.first.forget(uri) || self.second.forget(uri)
    }

    async fn read_to_string(&mut self, uri: &Url) -> Result<String, DriverError> {
        match self.first.read_to_string(uri).await {
            Ok(source) => Ok(source),
            Err(DriverError::FileNotFound(_)) => self.second.read_to_string(uri).await,
            Err(err) => Err(err),
        }
    }

    async fn write_string(&mut self, uri: &Url, source: &str) -> Result<(), DriverError> {
        if self.first.exists(uri).await? {
            self.first.write_string(uri, source).await
        } else {
            self.second.write_string(uri, source).await
        }
    }
}
