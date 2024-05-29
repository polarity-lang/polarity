pub use pretty::termcolor;
pub use pretty::termcolor::Color;
pub use pretty::termcolor::ColorChoice;
pub use pretty::termcolor::ColorSpec;
pub use pretty::termcolor::StandardStream;
pub use pretty::termcolor::WriteColor;
pub use pretty::DocAllocator;

mod ctx;
mod de_bruijn;
pub mod fragments;
mod generic;
mod print_to_string;
mod render;
pub mod theme;
pub mod tokens;
pub mod types;
pub mod util;

pub use print_to_string::*;
pub use types::*;

pub const DEFAULT_WIDTH: usize = 100;
