use ast::HashMap;
use decls::*;
use ident::Ident;
use parser::cst::*;

pub mod build;
pub mod lookup;

#[derive(Debug, Default, Clone)]
pub struct LookupTable {
    map: HashMap<Ident, DeclMeta>,
}

impl LookupTable {
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
