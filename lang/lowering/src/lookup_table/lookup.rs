use miette_util::ToMiette;
use parser::cst::ident::Ident;

use crate::LoweringError;

use super::{DeclMeta, LookupTable};

impl LookupTable {
    /// Check whether the identifier already exists in any of the symbol tables.
    pub fn lookup_exists(&self, name: &Ident) -> bool {
        for symbol_table in self.map.values() {
            if symbol_table.contains_key(name) {
                return true;
            }
        }
        false
    }

    pub fn lookup(&self, name: &Ident) -> Result<&DeclMeta, LoweringError> {
        for symbol_table in self.map.values() {
            match symbol_table.get(name) {
                Some(meta) => return Ok(meta),
                None => continue,
            }
        }
        Err(LoweringError::UndefinedIdent { name: name.clone(), span: name.span.to_miette() })
    }
}
