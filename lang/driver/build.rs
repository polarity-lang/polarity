use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    generate_js_runtime();
}

fn generate_js_runtime() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root_dir = manifest_dir.parent().unwrap().parent().unwrap();
    let runtime_dir = root_dir.join("runtime");

    let status = Command::new("npm")
        .args(["run", "build"])
        .current_dir(&runtime_dir)
        .status()
        .expect("failed to run npm");

    assert!(status.success());

    for file in ["runtime.ts", "runtime.js", "package.json", "package-lock.json", "tsconfig.json"] {
        println!("cargo:rerun-if-changed={}", runtime_dir.join(file).display())
    }
}
