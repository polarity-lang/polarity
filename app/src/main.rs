#[cfg(not(target_arch = "wasm32"))]
mod cli;
#[cfg(not(target_arch = "wasm32"))]
mod result;
#[cfg(not(target_arch = "wasm32"))]
mod rt;

pub const VERSION: &str = env!("VERSION");

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    cli::exec();
}

#[cfg(target_arch = "wasm32")]
fn main() {}
