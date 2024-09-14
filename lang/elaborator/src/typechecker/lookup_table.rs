use ast::*;

use super::TypeError;

#[derive(Debug, Clone, Default)]
pub struct LookupTable {
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

impl LookupTable {
    pub fn lookup_ctor_or_codef(&self, name: &Ident) -> Result<CtorMeta, TypeError> {
        self.map_ctor
            .get(name)
            .cloned()
            .or_else(|| self.map_codef.get(name).map(|codef| codef.to_ctor()))
            .ok_or(TypeError::Impossible {
                message: format!("Top-level ctor or codef {name} not found"),
                span: None,
            })
    }

    pub fn lookup_dtor_or_def(&self, name: &Ident) -> Result<DtorMeta, TypeError> {
        self.map_dtor
            .get(name)
            .cloned()
            .or_else(|| self.map_def.get(name).map(|def| def.to_dtor()))
            .ok_or(TypeError::Impossible {
                message: format!("Top-level dtor or def {name} not found"),
                span: None,
            })
    }

    pub fn lookup_let(&self, name: &Ident) -> Result<&LetMeta, TypeError> {
        self.map_let.get(name).ok_or(TypeError::Impossible {
            message: format!("Top-level let {name} not found"),
            span: None,
        })
    }

    pub fn lookup_tyctor(&self, name: &Ident) -> Result<&TyCtorMeta, TypeError> {
        self.map_tyctor.get(name).ok_or(TypeError::Impossible {
            message: format!("Top-level tyctor {name} not found"),
            span: None,
        })
    }

    pub fn lookup_codef(&self, name: &Ident) -> Result<&CodefMeta, TypeError> {
        self.map_codef.get(name).ok_or(TypeError::Impossible {
            message: format!("Top-level codef {name} not found"),
            span: None,
        })
    }

    pub fn lookup_ctor(&self, name: &Ident) -> Result<&CtorMeta, TypeError> {
        self.map_ctor.get(name).ok_or(TypeError::Impossible {
            message: format!("Top-level ctor {name} not found"),
            span: None,
        })
    }

    pub fn lookup_def(&self, name: &Ident) -> Result<&DefMeta, TypeError> {
        self.map_def.get(name).ok_or(TypeError::Impossible {
            message: format!("Top-level def {name} not found"),
            span: None,
        })
    }

    pub fn lookup_dtor(&self, name: &Ident) -> Result<&DtorMeta, TypeError> {
        self.map_dtor.get(name).ok_or(TypeError::Impossible {
            message: format!("Top-level dtor {name} not found"),
            span: None,
        })
    }

    pub fn append(&mut self, other: LookupTable) {
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

pub fn build_lookup_table(module: &Module) -> LookupTable {
    let mut lookup_table = LookupTable::default();

    let Module { decls, .. } = module;

    for decl in decls {
        match decl {
            Decl::Data(data) => build_data(&mut lookup_table, data),
            Decl::Codata(codata) => build_codata(&mut lookup_table, codata),
            Decl::Def(def) => build_def(&mut lookup_table, def),
            Decl::Codef(codef) => build_codef(&mut lookup_table, codef),
            Decl::Let(tl_let) => build_let(&mut lookup_table, tl_let),
        }
    }

    lookup_table
}

fn build_data(lookup_table: &mut LookupTable, data: &Data) {
    let Data { name, typ, ctors, .. } = data;
    lookup_table.map_tyctor.insert(name.clone(), TyCtorMeta { params: typ.clone() });
    for ctor in ctors {
        build_ctor(lookup_table, ctor);
    }
}

fn build_ctor(lookup_table: &mut LookupTable, ctor: &Ctor) {
    let Ctor { name, params, typ, .. } = ctor;
    lookup_table
        .map_ctor
        .insert(name.clone(), CtorMeta { params: params.clone(), typ: typ.clone() });
}

fn build_codata(lookup_table: &mut LookupTable, codata: &Codata) {
    let Codata { name, typ, dtors, .. } = codata;
    lookup_table.map_tyctor.insert(name.clone(), TyCtorMeta { params: typ.clone() });
    for dtor in dtors {
        build_dtor(lookup_table, dtor);
    }
}

fn build_dtor(lookup_table: &mut LookupTable, dtor: &Dtor) {
    let Dtor { name, params, self_param, ret_typ, .. } = dtor;
    lookup_table.map_dtor.insert(
        name.clone(),
        DtorMeta {
            params: params.clone(),
            self_param: self_param.clone(),
            ret_typ: ret_typ.clone(),
        },
    );
}

fn build_def(lookup_table: &mut LookupTable, def: &Def) {
    let Def { name, params, self_param, ret_typ, .. } = def;
    lookup_table.map_def.insert(
        name.clone(),
        DefMeta {
            params: params.clone(),
            self_param: self_param.clone(),
            ret_typ: ret_typ.clone(),
        },
    );
}

fn build_codef(lookup_table: &mut LookupTable, codef: &Codef) {
    let Codef { name, params, typ, .. } = codef;
    lookup_table
        .map_codef
        .insert(name.clone(), CodefMeta { params: params.clone(), typ: typ.clone() });
}

fn build_let(lookup_table: &mut LookupTable, tl_let: &Let) {
    let Let { name, params, typ, .. } = tl_let;
    lookup_table.map_let.insert(name.clone(), LetMeta { params: params.clone(), typ: typ.clone() });
}
