use ast::*;
use url::Url;

use super::TypeError;

pub mod build;
pub mod lookup;

#[derive(Debug, Clone, Default)]
pub struct TypeInfoTable {
    map: HashMap<Url, ModuleTypeInfoTable>,
}

impl TypeInfoTable {
    pub fn insert(&mut self, uri: Url, info_table: ModuleTypeInfoTable) {
        self.map.insert(uri, info_table);
    }
}

#[derive(Debug, Clone, Default)]
pub struct ModuleTypeInfoTable {
    // Data and Codata Types
    //
    //
    map_data: HashMap<String, Data>,
    map_codata: HashMap<String, Codata>,
    // Calls
    //
    //
    map_let: HashMap<String, Let>,
    map_tyctor: HashMap<String, TyCtorMeta>,
    map_codef: HashMap<String, Codef>,
    map_ctor: HashMap<String, CtorMeta>,
    // DotCalls
    //
    //
    map_def: HashMap<String, Def>,
    map_dtor: HashMap<String, DtorMeta>,
}

#[derive(Debug, Clone)]
pub struct TyCtorMeta {
    pub params: Box<Telescope>,
}

#[derive(Debug, Clone)]
pub struct CtorMeta {
    pub params: Telescope,
    pub typ: TypCtor,
}

impl From<Ctor> for CtorMeta {
    fn from(ctor: Ctor) -> Self {
        CtorMeta { params: ctor.params, typ: ctor.typ }
    }
}

#[derive(Debug, Clone)]
pub struct DtorMeta {
    pub params: Telescope,
    pub self_param: SelfParam,
    pub ret_typ: Box<Exp>,
}

impl From<Dtor> for DtorMeta {
    fn from(dtor: Dtor) -> Self {
        DtorMeta { params: dtor.params, self_param: dtor.self_param, ret_typ: dtor.ret_typ }
    }
}
