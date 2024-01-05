#[cfg(not(target_arch = "wasm32"))]
mod cli;
#[cfg(not(target_arch = "wasm32"))]
mod result;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> miette::Result<()> {
    miette::set_panic_hook();
    cli::exec()
}

#[cfg(target_arch = "wasm32")]
fn main() {}
