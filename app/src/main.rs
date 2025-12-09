#[cfg(not(target_arch = "wasm32"))]
mod cli;

#[cfg(not(target_arch = "wasm32"))]
mod global_settings;

mod utils;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    miette::set_panic_hook();

    // Get global settings from environment
    let mut settings = global_settings::GlobalSettings::from_env();

    // Run app
    let result = cli::exec(&mut settings);

    // Output any errors
    if let Err(errors) = result {
        let mut stderr = std::io::stderr().lock();
        polarity_lang_driver::render_reports_io(
            &mut stderr,
            &errors,
            settings.colorize != polarity_lang_printer::ColorChoice::Never,
        );
        std::process::exit(1);
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {}
