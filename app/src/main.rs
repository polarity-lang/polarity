#[cfg(not(target_arch = "wasm32"))]
mod cli;

mod utils;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), ()> {
    miette::set_panic_hook();
    let result = cli::exec();

    if let Err(errors) = result {
        for error in errors {
            eprintln!("{error:?}");
        }
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}
