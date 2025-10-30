#[cfg(not(target_arch = "wasm32"))]
mod cli;

mod utils;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    miette::set_panic_hook();
    let result = cli::exec();

    if let Err(errors) = result {
        for error in errors {
            eprintln!("{error:?}");
        }
        std::process::exit(1);
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {}
