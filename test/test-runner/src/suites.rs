use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use url::Url;
use walkdir::WalkDir;

use serde_derive::Deserialize;

// Case
//
// Individual testcases which are combined in a testsuite.

/// One individual testcase within a testsuite.
/// The testing semantics (i.e. whether the case should fail or succeed)
/// is determined by the testsuite of which it is a part.
#[derive(Clone, PartialEq)]
pub struct Case {
    /// The name of the testsuite to which this testcase belongs.
    pub suite: String,
    /// The name of this testcase.
    pub name: String,
    /// The path of the `<file>.pol` file.
    pub path: PathBuf,
}

impl Case {
    pub fn new(suite: String, path: PathBuf) -> Self {
        let name = path.file_stem().unwrap().to_str().unwrap().to_owned();

        Self { suite, name, path }
    }

    pub fn content(&self) -> io::Result<String> {
        // Depending on how git is configured on Windows, it may check-out Unix line endings (\n) as Windows line endings (\r\n).
        // If this is the case, we need to replace these by Unix line endings for comparision.
        fs::read_to_string(&self.path).map(|s| s.replace("\r\n", "\n"))
    }

    pub fn uri(&self) -> Url {
        let canonicalized_path = self.path.clone().canonicalize().unwrap();

        Url::from_file_path(canonicalized_path).unwrap()
    }

    pub fn expected(&self, phase_name: &str) -> Option<String> {
        let path = self.expected_path(phase_name);
        // Depending on how git is configured on Windows, it may check-out Unix line endings (\n) as Windows line endings (\r\n).
        // If this is the case, we need to replace these by Unix line endings for comparision.
        path.is_file().then(|| fs::read_to_string(path).unwrap().replace("\r\n", "\n"))
    }

    pub fn set_expected(&self, phase_name: &str, s: &str) {
        fs::write(self.expected_path(phase_name), s).unwrap();
    }

    fn expected_path(&self, phase_name: &str) -> PathBuf {
        let file_extension = phase_name.to_lowercase();
        let path =
            self.path.parent().unwrap().join(format!("{}.{}.expected", self.name, file_extension));
        if path.exists() {
            path
        } else {
            // Fallback: Tests that expect failure have a single expected file for the error message.
            self.path.parent().unwrap().join(format!("{}.expected", self.name))
        }
    }
}

// Suites
//
// A testsuite consisting of individual cases.

pub fn load<P: AsRef<Path>>(path: P) -> impl Iterator<Item = Suite> {
    let suite_paths = fs::read_dir(path).unwrap().filter_map(|entry| {
        let path = entry.unwrap().path();
        if path.is_dir() {
            Some(path)
        } else {
            None
        }
    });
    suite_paths.map(Suite::new)
}

/// Each testsuite is configured by a `suite.toml` file whose contents
/// are described by this struct.
#[derive(Default, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Config {
    /// In which phase the cases of the testsuite are supposed to fail.
    /// If this is none, then the testcases should succeed.
    pub fail: Option<String>,
    /// Human-readable description of what the tests in this suite are testing.
    pub description: String,
}

/// A single testsuite such as "fail-lower", "fail-check" or "success".
#[derive(Clone)]
pub struct Suite {
    /// The name of the testsuite.
    pub name: String,
    /// The parsed content of the `suite.toml` file.
    pub config: Config,
    /// The individual cases which belong to this testsuite.
    pub cases: Vec<Case>,
}

impl Suite {
    pub fn new(path: PathBuf) -> Self {
        // Compute the name of the testsuite
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();

        // Read in the configuration from the `suite.toml` file.
        let config_path = path.join("suite.toml");
        let config = if config_path.is_file() {
            let text = fs::read_to_string(config_path).unwrap();
            toml::from_str(&text).unwrap()
        } else {
            Config::default()
        };

        let mut cases: Vec<Case> = Vec::new();

        // Read in the cases which belong to this testsuite.
        // Every file in the path ending in `.pol` is a testcase.
        for entry in WalkDir::new(&path).follow_links(false).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension() == Some(OsStr::new("pol")) {
                cases.push(Case::new(name.to_owned(), path.to_path_buf()))
            }
        }

        Suite { name, config, cases }
    }
}
