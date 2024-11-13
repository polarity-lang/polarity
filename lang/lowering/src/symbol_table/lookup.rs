use miette_util::ToMiette;
use parser::cst::ident::Ident;
use url::Url;

use crate::LoweringError;

use super::{DeclMeta, SymbolTable};

impl SymbolTable {
    /// Check whether the identifier already exists in any of the symbol tables.
    pub fn lookup_exists(&self, name: &Ident) -> bool {
        for symbol_table in self.map.values() {
            if symbol_table.contains_key(name) {
                return true;
            }
        }
        false
    }

    pub fn lookup(&self, name: &Ident) -> Result<(&DeclMeta, &Url), LoweringError> {
        for (module_uri, symbol_table) in self.map.iter() {
            match symbol_table.get(name) {
                Some(meta) => return Ok((meta, module_uri)),
                None => continue,
            }
        }
        Err(LoweringError::UndefinedIdent { name: name.clone(), span: name.span.to_miette() })
    }
}
