use miette_util::ToMiette;
use parser::cst::ident::{Ident, Operator};
use url::Url;

use crate::LoweringError;

use super::{DeclMeta, SymbolTable};

impl SymbolTable {
    /// Check whether the identifier already exists in any of the symbol tables.
    pub fn lookup_exists(&self, name: &Ident) -> bool {
        for symbol_table in self.map.values() {
            if symbol_table.idents.contains_key(name) {
                return true;
            }
        }
        false
    }

    pub fn lookup(&self, name: &Ident) -> Result<(&DeclMeta, &Url), LoweringError> {
        for (module_uri, symbol_table) in self.map.iter() {
            match symbol_table.idents.get(name) {
                Some(meta) => return Ok((meta, module_uri)),
                None => continue,
            }
        }
        Err(LoweringError::UndefinedIdent { name: name.clone(), span: name.span.to_miette() })
    }

    /// Check whether the operator already exists in any of the symbol tables.
    pub fn lookup_operator_exists(&self, op: &Operator) -> bool {
        for symbol_table in self.map.values() {
            if symbol_table.infix_ops.contains_key(op) {
                return true;
            }
        }
        false
    }

    pub fn lookup_operator(&self, op: &Operator) -> Result<(&Ident, &Url), LoweringError> {
        for (module_uri, symbol_table) in self.map.iter() {
            match symbol_table.infix_ops.get(op) {
                Some(id) => return Ok((id, module_uri)),
                None => continue,
            }
        }
        Err(LoweringError::UnknownOperator { span: op.span.to_miette(), operator: op.id.clone() })
    }
}
