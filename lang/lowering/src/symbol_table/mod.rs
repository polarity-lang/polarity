use std::sync::Arc;

use decls::*;
use ident::Ident;
use polarity_lang_ast::HashMap;
use polarity_lang_parser::cst::{ident::Operator, *};
use url::Url;

pub mod build;
pub mod lookup;

/// The symbol table for a single module.
#[derive(Debug, Default, Clone)]
pub struct ModuleSymbolTable {
    /// The mapping of identifiers to their metadata
    pub idents: HashMap<Ident, DeclMeta>,
    /// The mapping of operators to their definition
    pub infix_ops: HashMap<Operator, Ident>,
}

/// The symbol table for a module and all of its imported modules.
#[derive(Debug, Default, Clone)]
pub struct SymbolTable {
    // Maps modules to their respective symbol tables.
    map: HashMap<Url, Arc<ModuleSymbolTable>>,
}

impl SymbolTable {
    pub fn insert(&mut self, url: Url, other: Arc<ModuleSymbolTable>) {
        self.map.insert(url, other);
    }
}

#[derive(Clone, Debug)]
pub enum DeclMeta {
    Data { params: Telescope },
    Codata { params: Telescope },
    Def { params: Telescope },
    Codef { params: Telescope },
    Ctor { params: Telescope },
    Dtor { params: Telescope },
    Let { params: Telescope },
    Note,
}
