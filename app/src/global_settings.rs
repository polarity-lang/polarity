use std::{env, str::FromStr};

use polarity_lang_printer::ColorChoice;

#[derive(Debug, Clone)]
pub struct GlobalSettings {
    pub colorize: ColorChoice,
    pub log_level: log::LevelFilter,
}

impl GlobalSettings {
    pub fn from_env() -> Self {
        let colorize = env::var("POLARITY_COLORIZE")
            .ok()
            .and_then(|var| ColorChoice::from_str(&var).ok())
            .unwrap_or(ColorChoice::Auto);

        let log_level = env::var("POLARITY_LOG_LEVEL")
            .ok()
            .and_then(|var| log::LevelFilter::from_str(&var.to_uppercase()).ok())
            .unwrap_or(log::LevelFilter::Info);

        Self { colorize, log_level }
    }
}
