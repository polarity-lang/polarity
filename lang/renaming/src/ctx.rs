use syntax::ast::*;
use syntax::common::*;
use syntax::ctx::{Context, ContextElem};

#[derive(Debug, Clone)]
pub struct Ctx {
    bound: Vec<Vec<Ident>>,
}

impl Context for Ctx {
    type ElemIn = Ident;

    type ElemOut = Ident;

    type Var = Idx;

    fn empty() -> Self {
        Self { bound: vec![] }
    }

    fn push_telescope(&mut self) {
        self.bound.push(vec![]);
    }

    fn pop_telescope(&mut self) {
        self.bound.pop().unwrap();
    }

    fn push_binder(&mut self, elem: Self::ElemIn) {
        self.bound.last_mut().expect("Cannot push without calling level_inc_fst first").push(elem);
    }

    fn pop_binder(&mut self, _elem: Self::ElemIn) {
        let err = "Cannot pop from empty context";
        self.bound.last_mut().expect(err).pop().expect(err);
    }

    fn lookup<V: Into<Self::Var> + std::fmt::Debug>(&self, var: V) -> Self::ElemOut {
        // FIXME: Handle shadowing
        let dbg: String = format!("{var:?}");
        let idx = var.into();
        self.bound
            .get(self.bound.len() - 1 - idx.fst)
            .and_then(|ctx| ctx.get(ctx.len() - 1 - idx.snd))
            .unwrap_or_else(|| panic!("Unbound variable: {dbg}, idx: {idx}"))
            .clone()
    }
}

impl<P: Phase> ContextElem<Ctx> for Param<P> {
    fn as_element(&self) -> <Ctx as Context>::ElemIn {
        self.name.to_owned()
    }
}

impl<P: Phase> ContextElem<Ctx> for ParamInst<P> {
    fn as_element(&self) -> <Ctx as Context>::ElemIn {
        self.name.to_owned()
    }
}
