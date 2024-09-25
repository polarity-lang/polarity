use ast::HashMap;
use decls::*;
use ident::Ident;
use parser::cst::*;
use url::Url;

pub mod build;
pub mod lookup;

/// The symbol table for a single module.
pub type ModuleSymbolTable = HashMap<Ident, DeclMeta>;

/// The symbol table for a module and all of its imported modules.
#[derive(Debug, Default, Clone)]
pub struct SymbolTable {
    // Maps modules to their respective symbol tables.
    map: HashMap<Url, ModuleSymbolTable>,
}

impl SymbolTable {
    pub fn insert(&mut self, url: Url, other: ModuleSymbolTable) {
        self.map.insert(url, other);
    }
    pub fn append(&mut self, other: SymbolTable) {
        self.map.extend(other.map);
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
}
