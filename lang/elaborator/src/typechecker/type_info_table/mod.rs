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
    map_let: HashMap<String, LetMeta>,
    map_tyctor: HashMap<String, TyCtorMeta>,
    map_codef: HashMap<String, CodefMeta>,
    map_ctor: HashMap<String, CtorMeta>,
    // DotCalls
    //
    //
    map_def: HashMap<String, DefMeta>,
    map_dtor: HashMap<String, DtorMeta>,
}

#[derive(Debug, Clone)]
pub struct LetMeta {
    pub params: Telescope,
    pub typ: Box<Exp>,
}

#[derive(Debug, Clone)]
pub struct TyCtorMeta {
    pub params: Box<Telescope>,
}

#[derive(Debug, Clone)]
pub struct CodefMeta {
    pub params: Telescope,
    pub typ: TypCtor,
}

impl CodefMeta {
    pub fn to_ctor(&self) -> CtorMeta {
        CtorMeta { params: self.params.clone(), typ: self.typ.clone() }
    }
}

#[derive(Debug, Clone)]
pub struct CtorMeta {
    pub params: Telescope,
    pub typ: TypCtor,
}

#[derive(Debug, Clone)]
pub struct DefMeta {
    pub params: Telescope,
    pub self_param: SelfParam,
    pub ret_typ: Box<Exp>,
}

impl DefMeta {
    fn to_dtor(&self) -> DtorMeta {
        DtorMeta {
            params: self.params.clone(),
            self_param: self.self_param.clone(),
            ret_typ: self.ret_typ.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DtorMeta {
    pub params: Telescope,
    pub self_param: SelfParam,
    pub ret_typ: Box<Exp>,
}
