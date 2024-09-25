use miette_util::ToMiette;
use parser::cst::ident::Ident;

use crate::LoweringError;

use super::{DeclMeta, LookupTable};

impl LookupTable {
    /// Check whether the identifier already exists.
    pub fn lookup_exists(&self, name: &Ident) -> bool {
        self.map.contains_key(name)
    }

    pub fn lookup(&self, name: &Ident) -> Result<&DeclMeta, LoweringError> {
        self.map.get(name).ok_or_else(|| LoweringError::UndefinedIdent {
            name: name.clone(),
            span: name.span.to_miette(),
        })
    }
}
