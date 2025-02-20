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

impl Zonk for ModuleTypeInfoTable {
    fn zonk(&mut self, meta_vars: &HashMap<MetaVar, MetaVarState>) -> Result<(), ZonkError> {
        let ModuleTypeInfoTable {
            map_data,
            map_codata,
            map_let,
            map_tyctor,
            map_codef,
            map_ctor,
            map_def,
            map_dtor,
        } = self;

        for (_, data) in map_data.iter_mut() {
            data.zonk(meta_vars)?;
        }

        for (_, codata) in map_codata.iter_mut() {
            codata.zonk(meta_vars)?;
        }

        for (_, let_) in map_let.iter_mut() {
            let_.zonk(meta_vars)?;
        }

        for (_, tyctor) in map_tyctor.iter_mut() {
            tyctor.params.zonk(meta_vars)?;
        }

        for (_, codef) in map_codef.iter_mut() {
            codef.zonk(meta_vars)?;
        }

        for (_, ctor) in map_ctor.iter_mut() {
            let CtorMeta { params, typ } = ctor;
            params.zonk(meta_vars)?;
            typ.zonk(meta_vars)?;
        }

        for (_, def) in map_def.iter_mut() {
            def.zonk(meta_vars)?;
        }

        for (_, dtor) in map_dtor.iter_mut() {
            let DtorMeta { params, self_param, ret_typ } = dtor;
            params.zonk(meta_vars)?;
            self_param.zonk(meta_vars)?;
            ret_typ.zonk(meta_vars)?;
        }

        Ok(())
    }
}
