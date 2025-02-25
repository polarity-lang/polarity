pub mod doc;
pub mod generate;
pub mod generate_docs;
pub mod printer;
pub mod util;
pub mod sidebar;

mod render;
pub use doc::write_html;
pub use printer::*;
pub use util::get_target_path;
pub use util::open;
pub use util::trim_windows_path_prefix;
pub use sidebar::generate_html_from_paths;
