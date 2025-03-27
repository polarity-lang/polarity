pub use pretty::DocAllocator;
pub use pretty::termcolor;
pub use pretty::termcolor::Color;
pub use pretty::termcolor::ColorChoice;
pub use pretty::termcolor::ColorSpec;
pub use pretty::termcolor::StandardStream;
pub use pretty::termcolor::WriteColor;

mod render;
pub mod theme;
pub mod tokens;
pub mod types;
pub mod util;

pub use types::*;

pub const DEFAULT_WIDTH: usize = 100;
