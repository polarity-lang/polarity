use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use syntax::ast;
use syntax::common::*;
use syntax::de_bruijn::*;
use syntax::var::*;

pub trait Lower {
    type Target;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError>;
}

pub trait LowerTelescope {
    type Target;

    fn lower_telescope<T, F: Fn(&mut Ctx, Self::Target) -> Result<T, LoweringError>>(
        &self,
        ctx: &mut Ctx,
        f: F,
    ) -> Result<T, LoweringError>;
}

impl<T: LowerTelescope> Lower for T {
    type Target = <Self as LowerTelescope>::Target;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        self.lower_telescope(ctx, |_, out| Ok(out))
    }
}

pub struct Ctx {
    /// For each name, store a vector representing the different binders
    /// represented by this name. The last entry represents the binder currently in scope,
    /// the remaining entries represent the binders which are currently shadowed.
    pub(super) map: HashMap<Ident, Vec<Var>>,
    /// Accumulates top-level declarations
    pub(super) decls: ast::Decls,
    /// Counts the number of binders currently in scope in the `lvl` part of the 2-level De Bruijn index
    pub(super) idx: Idx,
}

impl Ctx {
    pub fn empty() -> Self {
        Self { map: HashMap::new(), decls: ast::Decls::empty(), idx: Idx::here() }
    }

    pub fn lookup(&self, name: &Ident) -> Result<Var, LoweringError> {
        self.map
            .get(name)
            .and_then(|stack| stack.last())
            .map(|var| match var {
                Var::Bound(rev_idx) => Var::Bound(self.idx.map_lvl(|l| l - 1) - *rev_idx),
                v @ Var::Free(_) => v.clone(),
            })
            .ok_or_else(|| LoweringError::UndefinedIdent(name.clone()))
    }

    pub fn add_name(&mut self, name: &Ident) -> Result<(), LoweringError> {
        let var = Var::Free(name.clone());
        let stack = self.map.entry(name.clone()).or_insert_with(Default::default);
        stack.push(var);
        Ok(())
    }

    pub fn add_decl(&mut self, decl: ast::Decl) -> Result<(), LoweringError> {
        match self.decls.map.get(decl.name()) {
            Some(_) => Err(LoweringError::AlreadyDefined(decl.name().clone())),
            None => {
                self.decls.order.push(decl.name().clone());
                self.decls.map.insert(decl.name().clone(), decl);
                Ok(())
            }
        }
    }

    pub fn bind<T, F: FnOnce(&mut Ctx) -> T>(&mut self, name: Ident, f: F) -> T {
        self.push(name.clone());
        let res = f(self);
        self.pop(&name);
        res
    }

    fn push(&mut self, name: Ident) {
        let var = Var::Bound(self.idx);
        self.idx = self.idx.map_lvl(|l| l + 1);
        let stack = self.map.entry(name).or_insert_with(Default::default);
        stack.push(var);
    }

    fn pop(&mut self, name: &Ident) {
        let stack = self.map.get_mut(name).expect("Tried to read unknown variable");
        self.idx = self.idx.map_lvl(|l| l - 1);
        stack.pop().unwrap();
    }
}

#[derive(Debug)]
pub enum LoweringError {
    UndefinedIdent(Ident),
    AlreadyDefined(Ident),
}

impl fmt::Display for LoweringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UndefinedIdent(ident) => write!(f, "Undefined identifier {}", ident),
            Self::AlreadyDefined(ident) => write!(f, "Duplicate definition of {}", ident),
        }
    }
}

impl Error for LoweringError {}
