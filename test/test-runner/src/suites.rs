use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_derive::Deserialize;

// Case
//
//

pub fn load_case<'a, P: AsRef<Path> + 'a>(
    suite: &'a str,
    path: P,
) -> impl Iterator<Item = Case> + 'a {
    case_paths(path).map(|path| Case::new(suite.to_owned(), path))
}

#[derive(Clone)]
pub struct Case {
    pub suite: String,
    pub name: String,
    pub path: PathBuf,
}

impl Case {
    pub fn new(suite: String, path: PathBuf) -> Self {
        let name = path.file_stem().unwrap().to_str().unwrap().to_owned();

        Self { suite, name, path }
    }

    pub fn content(&self) -> io::Result<String> {
        fs::read_to_string(&self.path)
    }

    pub fn expected(&self) -> Option<String> {
        let path = self.expected_path();
        path.is_file().then(|| fs::read_to_string(path).unwrap())
    }

    pub fn set_expected(&self, s: &str) {
        fs::write(self.expected_path(), s).unwrap();
    }

    fn expected_path(&self) -> PathBuf {
        self.path.parent().unwrap().join(format!("{}.expected", self.name))
    }
}

fn case_paths<P: AsRef<Path>>(path: P) -> impl Iterator<Item = PathBuf> {
    fs::read_dir(path).unwrap().filter_map(|entry| {
        let path = entry.unwrap().path();
        if path.is_file() && path.extension() == Some(OsStr::new("pol")) {
            Some(path)
        } else {
            None
        }
    })
}

// Suites
//
//

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
    /// The directory in which the `suite.toml` for this testsuite is located.
    pub path: PathBuf,
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
        // Read in the cases which belong to this testsuite.
        let cases: Vec<Case> = load_case(&name, &path).collect();

        Suite { name, path, config, cases }
    }
}
