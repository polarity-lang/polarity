use parser::cst::ident::Ident;

use super::{DeclMeta, LookupTable};

impl LookupTable {
    pub fn lookup(&self, name: &Ident) -> Option<&DeclMeta> {
        self.map.get(name)
    }
}
