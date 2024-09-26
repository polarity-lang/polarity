use ast::*;

use super::{CodefMeta, CtorMeta, DefMeta, DtorMeta, LetMeta, TyCtorMeta, ModuleTypeInfoTable};

pub fn build_type_info_table(module: &Module) -> ModuleTypeInfoTable {
    let mut info_table = ModuleTypeInfoTable::default();

    let Module { decls, .. } = module;
    for decl in decls {
        decl.build(&mut info_table);
    }
    info_table
}

trait BuildTypeInfoTable {
    fn build(&self, info_table: &mut ModuleTypeInfoTable);
}

impl BuildTypeInfoTable for Decl {
    fn build(&self, info_table: &mut ModuleTypeInfoTable) {
        match self {
            Decl::Data(data) => data.build(info_table),
            Decl::Codata(codata) => codata.build(info_table),
            Decl::Def(def) => def.build(info_table),
            Decl::Codef(codef) => codef.build(info_table),
            Decl::Let(tl_let) => tl_let.build(info_table),
        }
    }
}

impl BuildTypeInfoTable for Data {
    fn build(&self, info_table: &mut ModuleTypeInfoTable) {
        let Data { name, typ, ctors, .. } = self;
        info_table.map_tyctor.insert(name.clone(), TyCtorMeta { params: typ.clone() });
        for ctor in ctors {
            ctor.build(info_table);
        }
    }
}

impl BuildTypeInfoTable for Ctor {
    fn build(&self, info_table: &mut ModuleTypeInfoTable) {
        let Ctor { name, params, typ, .. } = self;
        info_table
            .map_ctor
            .insert(name.clone(), CtorMeta { params: params.clone(), typ: typ.clone() });
    }
}

impl BuildTypeInfoTable for Codata {
    fn build(&self, info_table: &mut ModuleTypeInfoTable) {
        let Codata { name, typ, dtors, .. } = self;
        info_table.map_tyctor.insert(name.clone(), TyCtorMeta { params: typ.clone() });
        for dtor in dtors {
            dtor.build(info_table);
        }
    }
}

impl BuildTypeInfoTable for Dtor {
    fn build(&self, info_table: &mut ModuleTypeInfoTable) {
        let Dtor { name, params, self_param, ret_typ, .. } = self;
        info_table.map_dtor.insert(
            name.clone(),
            DtorMeta {
                params: params.clone(),
                self_param: self_param.clone(),
                ret_typ: ret_typ.clone(),
            },
        );
    }
}

impl BuildTypeInfoTable for Def {
    fn build(&self, info_table: &mut ModuleTypeInfoTable) {
        let Def { name, params, self_param, ret_typ, .. } = self;
        info_table.map_def.insert(
            name.clone(),
            DefMeta {
                params: params.clone(),
                self_param: self_param.clone(),
                ret_typ: ret_typ.clone(),
            },
        );
    }
}

impl BuildTypeInfoTable for Codef {
    fn build(&self, info_table: &mut ModuleTypeInfoTable) {
        let Codef { name, params, typ, .. } = self;
        info_table
            .map_codef
            .insert(name.clone(), CodefMeta { params: params.clone(), typ: typ.clone() });
    }
}

impl BuildTypeInfoTable for Let {
    fn build(&self, info_table: &mut ModuleTypeInfoTable) {
        let Let { name, params, typ, .. } = self;
        info_table
            .map_let
            .insert(name.clone(), LetMeta { params: params.clone(), typ: typ.clone() });
    }
}
