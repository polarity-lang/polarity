use std::fs;
use std::path::{Path, PathBuf};

use serde_derive::Deserialize;

use super::cases::{self, Case};

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
        cases::load(&self.name, &self.path)
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
