use parser::cst::ident::Ident;

use super::{DeclMeta, LookupTable};

impl LookupTable {
    /// Check whether the identifier already exists.
    pub fn lookup_exists(&self, name: &Ident) -> bool {
        self.map.contains_key(name)
    }

    pub fn lookup(&self, name: &Ident) -> Option<&DeclMeta> {
        self.map.get(name)
    }
}
