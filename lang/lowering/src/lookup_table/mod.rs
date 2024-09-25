use ast::HashMap;
use decls::*;
use ident::Ident;
use parser::cst::*;
use url::Url;

pub mod build;
pub mod lookup;

/// The lookup table for a single module.
pub type ModuleLookupTable = HashMap<Ident, DeclMeta>;

/// The lookup table for a module and all of its imported modules.
#[derive(Debug, Default, Clone)]
pub struct LookupTable {
    // Maps modules to their respective symbol tables.
    map: HashMap<Url, ModuleLookupTable>,
}

impl LookupTable {
    pub fn insert(&mut self, url: Url, other: ModuleLookupTable) {
        self.map.insert(url, other);
    }
    pub fn append(&mut self, other: LookupTable) {
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
