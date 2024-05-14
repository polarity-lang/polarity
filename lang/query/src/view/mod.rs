use url::Url;

mod edit;
mod lift;
mod rt;
mod spans;
mod xfunc;

pub use self::xfunc::*;
pub use edit::*;

use crate::*;

/// View on a file in the database
pub struct DatabaseView<'a> {
    pub(crate) url: Url,
    pub(crate) database: &'a Database,
}
