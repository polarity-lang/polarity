use ast::ctx::values::{Binder, TypeCtx};
use ast::ctx::{map_idx::*, GenericCtx, LevelCtx};
use ast::Idx;

use crate::normalizer::val::*;

pub type Env = GenericCtx<Box<Val>>;

pub trait ToEnv {
    fn env(&self) -> Env;
}

impl ToEnv for LevelCtx {
    fn env(&self) -> Env {
        let bound: Vec<_> = self
            .bound
            .iter()
            .enumerate()
            .map(|(fst, binders)| {
                binders
                    .iter()
                    .enumerate()
                    .map(|(snd, binder)| {
                        let idx =
                            Idx { fst: self.bound.len() - 1 - fst, snd: binders.len() - 1 - snd };
                        Binder {
                            name: binder.name.clone(),
                            content: Box::new(Val::Neu(Neu::Variable(Variable {
                                span: None,
                                name: binder.name.clone().into(),
                                idx,
                            }))),
                        }
                    })
                    .collect()
            })
            .collect();

        Env::from(bound)
    }
}

impl ToEnv for TypeCtx {
    fn env(&self) -> Env {
        let bound = self
            .bound
            .map_idx(|idx, binder| {
                Binder {
                    name: binder.name.clone().into(),
                    content: Box::new(Val::Neu(Neu::Variable(Variable {
                        // FIXME: handle info
                        span: None,
                        name: binder.name.clone().into(),
                        idx,
                    }))),
                }
            })
            .collect();

        Env::from(bound)
    }
}
