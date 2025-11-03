#[cfg(not(target_arch = "wasm32"))]
mod cli;

mod utils;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    miette::set_panic_hook();
    let result = cli::exec();

    if let Err(errors) = result {
        let mut stderr = std::io::stderr().lock();
        polarity_lang_driver::render_reports_io(&mut stderr, &errors, true);
        std::process::exit(1);
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {}
