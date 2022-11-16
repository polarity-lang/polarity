use std::process::{Command, Stdio};

fn get_version_string() -> String {
    let child = Command::new("git")
        .stdout(Stdio::piped())
        .args(["describe", "--tags", "--always"])
        .spawn()
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.code() == Some(0));
    String::from_utf8(output.stdout).unwrap()
}

fn main() {
    let version = get_version_string();
    println!("cargo:rustc-env=VERSION={}", version);
}
