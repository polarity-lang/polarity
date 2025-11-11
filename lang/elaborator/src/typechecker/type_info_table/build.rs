use polarity_lang_ast::*;

use crate::typechecker::erasure;

use super::{CtorMeta, DtorMeta, ModuleTypeInfoTable, TyCtorMeta};

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
            Decl::Extern(_) => todo!(),
            Decl::Infix(infix) => infix.build(info_table),
            Decl::Note(note) => note.build(info_table),
        }
    }
}

impl BuildTypeInfoTable for Data {
    fn build(&self, info_table: &mut ModuleTypeInfoTable) {
        info_table.map_data.insert(self.name.id.clone(), self.clone());
        let Data { name, typ, ctors, .. } = self;
        info_table.map_tyctor.insert(name.id.clone(), TyCtorMeta { params: typ.clone() });
        for ctor in ctors {
            ctor.build(info_table);
        }
    }
}

impl BuildTypeInfoTable for Ctor {
    fn build(&self, info_table: &mut ModuleTypeInfoTable) {
        let Ctor { name, params, typ, .. } = self;
        let mut params = params.clone();
        erasure::mark_erased_params(&mut params);
        info_table.map_ctor.insert(name.id.clone(), CtorMeta { params, typ: typ.clone() });
    }
}

impl BuildTypeInfoTable for Codata {
    fn build(&self, info_table: &mut ModuleTypeInfoTable) {
        info_table.map_codata.insert(self.name.id.clone(), self.clone());
        let Codata { name, typ, dtors, .. } = self;
        info_table.map_tyctor.insert(name.id.clone(), TyCtorMeta { params: typ.clone() });
        for dtor in dtors {
            dtor.build(info_table);
        }
    }
}

impl BuildTypeInfoTable for Dtor {
    fn build(&self, info_table: &mut ModuleTypeInfoTable) {
        let Dtor { name, params, self_param, ret_typ, .. } = self;
        let mut params = params.clone();
        erasure::mark_erased_params(&mut params);
        info_table.map_dtor.insert(
            name.id.clone(),
            DtorMeta { params, self_param: self_param.clone(), ret_typ: ret_typ.clone() },
        );
    }
}

impl BuildTypeInfoTable for Def {
    fn build(&self, info_table: &mut ModuleTypeInfoTable) {
        let mut def = self.clone();
        erasure::mark_erased_params(&mut def.params);
        info_table.map_def.insert(self.name.id.clone(), def);
    }
}

impl BuildTypeInfoTable for Codef {
    fn build(&self, info_table: &mut ModuleTypeInfoTable) {
        let mut codef = self.clone();
        erasure::mark_erased_params(&mut codef.params);
        info_table.map_codef.insert(self.name.id.clone(), codef);
    }
}

impl BuildTypeInfoTable for Let {
    fn build(&self, info_table: &mut ModuleTypeInfoTable) {
        let mut tl_let = self.clone();
        erasure::mark_erased_params(&mut tl_let.params);
        info_table.map_let.insert(self.name.id.clone(), tl_let);
    }
}

impl BuildTypeInfoTable for Infix {
    fn build(&self, _info_table: &mut ModuleTypeInfoTable) {}
}

impl BuildTypeInfoTable for Note {
    fn build(&self, _info_table: &mut ModuleTypeInfoTable) {}
}
