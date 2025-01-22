use std::fmt;

use ctx::values::Binder;

use crate::*;

use super::def::*;

// use data::string::comma_separated;
fn comma_separated<I: IntoIterator<Item = String>>(iter: I) -> String {
    separated(", ", iter)
}
fn separated<I: IntoIterator<Item = String>>(s: &str, iter: I) -> String {
    let vec: Vec<_> = iter.into_iter().collect();
    vec.join(s)
}

pub type LevelCtx = GenericCtx<()>;

impl LevelCtx {
    pub fn append(&self, other: &LevelCtx) -> Self {
        let mut bound = self.bound.clone();
        bound.extend(other.bound.iter().cloned());
        Self { bound }
    }

    pub fn tail(&self, skip: usize) -> Self {
        Self { bound: self.bound.iter().skip(skip).cloned().collect() }
    }

    // Swap the given indices
    pub fn swap(&self, fst1: usize, fst2: usize) -> Self {
        let mut new_ctx = self.clone();
        new_ctx.bound.swap(fst1, fst2);
        new_ctx
    }
}

impl From<Vec<Vec<Param>>> for LevelCtx {
    fn from(value: Vec<Vec<Param>>) -> Self {
        let bound = value
            .into_iter()
            .map(|v| v.into_iter().map(|p| Binder { name: p.name, content: () }).collect())
            .collect();
        Self { bound }
    }
}

impl Context for LevelCtx {
    type Elem = Binder<()>;

    fn push_telescope(&mut self) {
        self.bound.push(Vec::new());
    }

    fn pop_telescope(&mut self) {
        self.bound.pop().unwrap();
    }

    fn push_binder(&mut self, elem: Self::Elem) {
        self.bound.last_mut().expect("Cannot push without calling level_inc_fst first").push(elem);
    }

    fn pop_binder(&mut self, _elem: Self::Elem) {
        let err = "Cannot pop from empty context";
        self.bound.last_mut().expect(err).pop();
    }

    fn lookup<V: Into<Var>>(&self, idx: V) -> Self::Elem {
        let lvl = self.var_to_lvl(idx.into());
        self.bound
            .get(lvl.fst)
            .and_then(|ctx| ctx.get(lvl.snd))
            .unwrap_or_else(|| panic!("Unbound variable {lvl}"))
            .clone()
    }
}

impl From<Vec<Vec<VarBind>>> for LevelCtx {
    fn from(value: Vec<Vec<VarBind>>) -> Self {
        let bound = value
            .into_iter()
            .map(|v| v.into_iter().map(|b| Binder { name: b.clone(), content: () }).collect())
            .collect();
        Self { bound }
    }
}

impl ContextElem<LevelCtx> for VarBind {
    fn as_element(&self) -> <LevelCtx as Context>::Elem {
        Binder { name: self.clone(), content: () }
    }
}

impl ContextElem<LevelCtx> for &Binder<()> {
    fn as_element(&self) -> <LevelCtx as Context>::Elem {
        Binder { name: self.name.clone(), content: () }
    }
}

impl ContextElem<LevelCtx> for &Param {
    fn as_element(&self) -> <LevelCtx as Context>::Elem {
        Binder { name: self.name.clone(), content: () }
    }
}

impl ContextElem<LevelCtx> for &ParamInst {
    fn as_element(&self) -> <LevelCtx as Context>::Elem {
        Binder { name: self.name.clone(), content: () }
    }
}

impl ContextElem<LevelCtx> for &Option<Motive> {
    fn as_element(&self) -> <LevelCtx as Context>::Elem {
        let name = match self {
            Some(m) => m.param.name.clone(),
            None => VarBind::Wildcard { span: None },
        };
        Binder { name, content: () }
    }
}

impl ContextElem<LevelCtx> for &SelfParam {
    fn as_element(&self) -> <LevelCtx as Context>::Elem {
        let name = self.name.clone().unwrap_or(VarBind::Wildcard { span: None });
        Binder { name, content: () }
    }
}

impl fmt::Display for LevelCtx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", comma_separated(self.bound.iter().map(|v| v.len().to_string())))
    }
}
