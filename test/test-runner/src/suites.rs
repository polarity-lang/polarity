use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_derive::Deserialize;

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
    suite_paths(path).map(Suite::new)
}

#[derive(Clone)]
pub struct Suite {
    pub name: String,
    pub path: PathBuf,
}

impl Suite {
    pub fn new(path: PathBuf) -> Self {
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();
        Suite { name, path }
    }

    pub fn cases(&self) -> impl Iterator<Item = Case> + '_ {
        load_case(&self.name, &self.path)
    }

    pub fn config(&self) -> Config {
        let path = self.path.join("suite.toml");
        if path.is_file() {
            let text = fs::read_to_string(path).unwrap();
            toml::from_str(&text).unwrap()
        } else {
            Config::default()
        }
    }
}

fn suite_paths<P: AsRef<Path>>(path: P) -> impl Iterator<Item = PathBuf> {
    fs::read_dir(path).unwrap().filter_map(|entry| {
        let path = entry.unwrap().path();
        if path.is_dir() {
            Some(path)
        } else {
            None
        }
    })
}

#[derive(Default, Deserialize)]
pub struct Config {
    pub fail: Option<String>,
}
