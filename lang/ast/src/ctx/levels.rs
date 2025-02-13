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

impl From<Vec<Vec<VarBind>>> for LevelCtx {
    fn from(value: Vec<Vec<VarBind>>) -> Self {
        let bound = value
            .into_iter()
            .map(|v| v.into_iter().map(|b| Binder { name: b.clone(), content: () }).collect())
            .collect();
        Self { bound }
    }
}

impl AsBinder<()> for VarBind {
    fn as_binder(&self) -> Binder<()> {
        Binder { name: self.clone(), content: () }
    }
}

impl AsBinder<()> for &Binder<()> {
    fn as_binder(&self) -> Binder<()> {
        Binder { name: self.name.clone(), content: () }
    }
}

impl AsBinder<()> for &Param {
    fn as_binder(&self) -> Binder<()> {
        Binder { name: self.name.clone(), content: () }
    }
}

impl AsBinder<()> for &ParamInst {
    fn as_binder(&self) -> Binder<()> {
        Binder { name: self.name.clone(), content: () }
    }
}

impl AsBinder<()> for &Option<Motive> {
    fn as_binder(&self) -> Binder<()> {
        let name = match self {
            Some(m) => m.param.name.clone(),
            None => VarBind::Wildcard { span: None },
        };
        Binder { name, content: () }
    }
}

impl AsBinder<()> for &SelfParam {
    fn as_binder(&self) -> Binder<()> {
        Binder { name: self.name.clone(), content: () }
    }
}

impl fmt::Display for LevelCtx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", comma_separated(self.bound.iter().map(|v| v.len().to_string())))
    }
}
