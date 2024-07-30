use syntax::ast::*;
use syntax::ctx::{Context, ContextElem};

use super::util::increment_name;

#[derive(Debug, Clone)]
pub struct Ctx {
    bound: Vec<Vec<Ident>>,
}

impl Context for Ctx {
    type Elem = Ident;

    fn push_telescope(&mut self) {
        self.bound.push(vec![]);
    }

    fn pop_telescope(&mut self) {
        self.bound.pop().unwrap();
    }

    fn push_binder(&mut self, elem: Self::Elem) {
        assert!(elem == "_" || elem.is_empty() || !self.contains_name(&elem));
        self.bound.last_mut().expect("Cannot push without calling level_inc_fst first").push(elem);
    }

    fn pop_binder(&mut self, _elem: Self::Elem) {
        let err = "Cannot pop from empty context";
        self.bound.last_mut().expect(err).pop().expect(err);
    }

    fn lookup<V: Into<Var>>(&self, idx: V) -> Self::Elem {
        let lvl = self.var_to_lvl(idx.into());
        self.bound
            .get(lvl.fst)
            .and_then(|ctx| ctx.get(lvl.snd))
            .unwrap_or_else(|| panic!("Unbound variable {lvl}"))
            .clone()
    }

    fn idx_to_lvl(&self, idx: Idx) -> Lvl {
        let fst = self.bound.len() - 1 - idx.fst;
        let snd = self.bound[fst].len() - 1 - idx.snd;
        Lvl { fst, snd }
    }

    fn lvl_to_idx(&self, lvl: Lvl) -> Idx {
        let fst = self.bound.len() - 1 - lvl.fst;
        let snd = self.bound[lvl.fst].len() - 1 - lvl.snd;
        Idx { fst, snd }
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
    fn as_element(&self) -> <Ctx as Context>::Elem {
        self.name.to_owned()
    }
}

impl ContextElem<Ctx> for ParamInst {
    fn as_element(&self) -> <Ctx as Context>::Elem {
        self.name.to_owned()
    }
}

impl ContextElem<Ctx> for SelfParam {
    fn as_element(&self) -> <Ctx as Context>::Elem {
        self.name.to_owned().unwrap_or_default()
    }
}
