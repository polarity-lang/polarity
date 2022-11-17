//! Program context
//!
//! Tracks global names

use std::rc::Rc;

use data::{HashMap, HashSet};
use syntax::common::*;
use syntax::ust;

#[derive(Debug, Clone)]
pub struct Prg {
    types: HashMap<Ident, Rc<ust::TypAbs>>,
    ctors: HashMap<Ident, Rc<ust::Ctor>>,
    dtors: HashMap<Ident, Rc<ust::Dtor>>,
    type_for_xtor: HashMap<Ident, Ident>,
    xtors_in_type: HashMap<Ident, HashSet<Ident>>,
}

impl Prg {
    pub fn build(prg: &ust::Prg) -> Self {
        let mut types = HashMap::default();
        let mut ctors = HashMap::default();
        let mut dtors = HashMap::default();
        let mut type_for_xtor = HashMap::default();
        let mut xtors_in_type = HashMap::default();
        let mut xdefs_in_type: HashMap<Ident, HashSet<Ident>> = HashMap::default();

        for (decl_name, decl) in prg.decls.map.iter() {
            match decl {
                ust::Decl::Data(data) => {
                    types.insert(decl_name.clone(), data.typ.clone());
                    let mut xtors_set = HashSet::default();
                    for ctor in &data.ctors {
                        xtors_set.insert(ctor.clone());
                        type_for_xtor.insert(ctor.clone(), decl_name.clone());
                    }
                    xtors_in_type.insert(decl_name.clone(), xtors_set);
                }
                ust::Decl::Codata(codata) => {
                    types.insert(decl_name.clone(), codata.typ.clone());
                    let mut xtors_set = HashSet::default();
                    for dtor in &codata.dtors {
                        xtors_set.insert(dtor.clone());
                        type_for_xtor.insert(dtor.clone(), decl_name.clone());
                    }
                    xtors_in_type.insert(decl_name.clone(), xtors_set);
                }
                ust::Decl::Ctor(ctor) => {
                    ctors.insert(ctor.name.clone(), Rc::new(ctor.clone()));
                }
                ust::Decl::Dtor(dtor) => {
                    dtors.insert(dtor.name.clone(), Rc::new(dtor.clone()));
                }
                ust::Decl::Def(def) => {
                    dtors.insert(def.name.clone(), Rc::new(def.to_dtor()));
                    type_for_xtor.insert(def.name.clone(), def.on_typ.name.clone());
                    xdefs_in_type
                        .entry(def.on_typ.name.clone())
                        .or_default()
                        .insert(def.name.clone());
                }
                ust::Decl::Codef(codef) => {
                    ctors.insert(codef.name.clone(), Rc::new(codef.to_ctor()));
                    type_for_xtor.insert(codef.name.clone(), codef.typ.name.clone());
                    xdefs_in_type
                        .entry(codef.typ.name.clone())
                        .or_default()
                        .insert(codef.name.clone());
                }
            }
        }

        Self { types, ctors, dtors, type_for_xtor, xtors_in_type }
    }

    pub fn typ_for_xtor(&self, name: &Ident) -> &Ident {
        &self.type_for_xtor[name]
    }

    pub fn xtors_for_typ(&self, name: &Ident) -> &HashSet<Ident> {
        &self.xtors_in_type[name]
    }

    pub fn typ(&self, name: &Ident) -> Rc<ust::TypAbs> {
        self.types[name].clone()
    }

    pub fn ctor(&self, name: &Ident) -> Rc<ust::Ctor> {
        self.ctors[name].clone()
    }

    pub fn dtor(&self, name: &Ident) -> Rc<ust::Dtor> {
        self.dtors[name].clone()
    }
}
