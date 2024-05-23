#[cfg(not(target_arch = "wasm32"))]
mod cli;
#[cfg(not(target_arch = "wasm32"))]
mod result;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> miette::Result<()> {
    env_logger::builder().format_timestamp(None).format_level(false).format_target(false).init();
    miette::set_panic_hook();
    cli::exec()
}

#[cfg(target_arch = "wasm32")]
fn main() {}
