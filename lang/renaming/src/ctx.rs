use syntax::ast::*;
use syntax::ctx::{Context, ContextElem};

use super::util::increment_name;

#[derive(Debug, Clone)]
pub struct Ctx {
    bound: Vec<Vec<Ident>>,
}

impl Context for Ctx {
    type ElemIn = Ident;

    type ElemOut = Ident;

    type Var = Idx;

    fn push_telescope(&mut self) {
        self.bound.push(vec![]);
    }

    fn pop_telescope(&mut self) {
        self.bound.pop().unwrap();
    }

    fn push_binder(&mut self, elem: Self::ElemIn) {
        assert!(elem == "_" || elem.is_empty() || !self.contains_name(&elem));
        self.bound.last_mut().expect("Cannot push without calling level_inc_fst first").push(elem);
    }

    fn pop_binder(&mut self, _elem: Self::ElemIn) {
        let err = "Cannot pop from empty context";
        self.bound.last_mut().expect(err).pop().expect(err);
    }

    fn lookup<V: Into<Self::Var> + std::fmt::Debug>(&self, var: V) -> Self::ElemOut {
        let dbg: String = format!("{var:?}");
        let idx = var.into();
        self.bound
            .get(self.bound.len() - 1 - idx.fst)
            .and_then(|ctx| ctx.get(ctx.len() - 1 - idx.snd))
            .unwrap_or_else(|| panic!("Unbound variable: {dbg}, idx: {idx}"))
            .clone()
    }
}

impl Ctx {
    pub(super) fn disambiguate_name(&self, mut name: Ident) -> Ident {
        if name == "_" || name.is_empty() {
            "x".clone_into(&mut name);
        }
        while self.contains_name(&name) {
            name = increment_name(name);
        }
        name
    }

    pub fn empty() -> Self {
        Self { bound: vec![] }
    }

    fn contains_name(&self, name: &Ident) -> bool {
        for telescope in &self.bound {
            if telescope.contains(name) {
                return true;
            }
        }
        false
    }
}

impl ContextElem<Ctx> for Param {
    fn as_element(&self) -> <Ctx as Context>::ElemIn {
        self.name.to_owned()
    }
}

impl ContextElem<Ctx> for ParamInst {
    fn as_element(&self) -> <Ctx as Context>::ElemIn {
        self.name.to_owned()
    }
}

impl ContextElem<Ctx> for SelfParam {
    fn as_element(&self) -> <Ctx as Context>::ElemIn {
        self.name.to_owned().unwrap_or_default()
    }
}
