use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Entry {
    name: String,
    path: String,
    target: String,
}

pub fn read_json(json: &Path) -> Vec<Entry> {
    let mut file = File::open(json).expect("Failed to open file");
    let mut data = String::new();
    file.read_to_string(&mut data).expect("Failed to read file");
    serde_json::from_str(&data).expect("Failed to parse JSON")
}

fn entry_to_html(entry: &Entry) -> String {
    format!("<li><a href={}>{}</a></li>", &entry.target.split('/').last().unwrap().to_string(), entry.name)
}

pub fn list_to_html(entries: &[Entry]) -> String {
    let mut html = String::new();
    for entry in entries {
        html.push_str(&entry_to_html(entry));
    }
    html
}

pub fn get_name(entry: &Entry) -> String {
    entry.name.clone()
}

pub fn get_path(entry: &Entry) -> String {
    entry.path.clone()
}

pub fn get_target(entry: &Entry) -> String {
    entry.target.clone()
}