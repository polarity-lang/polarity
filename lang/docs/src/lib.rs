pub mod doc;
pub mod generate;
pub mod generate_docs;
pub mod printer;
pub mod util;

mod render;
pub use doc::write_html;
pub use printer::*;
pub use util::get_target_path;
pub use util::open;
