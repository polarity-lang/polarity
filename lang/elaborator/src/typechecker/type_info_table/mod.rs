use ast::*;

use super::TypeError;

pub mod build;
pub mod lookup;

#[derive(Debug, Clone, Default)]
pub struct ModuleTypeInfoTable {
    // Calls
    //
    //
    map_let: HashMap<Ident, LetMeta>,
    map_tyctor: HashMap<Ident, TyCtorMeta>,
    map_codef: HashMap<Ident, CodefMeta>,
    map_ctor: HashMap<Ident, CtorMeta>,
    // DotCalls
    //
    //
    map_def: HashMap<Ident, DefMeta>,
    map_dtor: HashMap<Ident, DtorMeta>,
}

impl ModuleTypeInfoTable {
    pub fn append(&mut self, other: ModuleTypeInfoTable) {
        self.map_let.extend(other.map_let);
        self.map_tyctor.extend(other.map_tyctor);
        self.map_codef.extend(other.map_codef);
        self.map_ctor.extend(other.map_ctor);
        self.map_def.extend(other.map_def);
        self.map_dtor.extend(other.map_dtor);
    }
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

