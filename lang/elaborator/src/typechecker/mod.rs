pub mod ctx;
pub mod decls;
pub mod exprs;
pub mod lookup_table;
pub mod util;

pub use crate::result::TypeError;
pub use decls::check;
pub use decls::check_with_lookup_table;
