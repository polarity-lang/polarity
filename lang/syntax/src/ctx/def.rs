//! Generic definition of variable contexts

use derivative::Derivative;

use crate::ast::{Idx, Lvl, Var};

use super::LevelCtx;

#[derive(Debug, Clone, Default, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct GenericCtx<T> {
    pub bound: Vec<Vec<T>>,
}

impl<T> GenericCtx<T> {
    pub fn len(&self) -> usize {
        self.bound.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bound.is_empty()
    }

    pub fn empty() -> Self {
        Self { bound: vec![] }
    }

    pub fn levels(&self) -> LevelCtx {
        let bound: Vec<_> = self.bound.iter().map(|inner| inner.len()).collect();
        LevelCtx::from(bound)
    }

    pub fn iter(&self) -> impl Iterator<Item = &[T]> {
        self.bound.iter().map(|inner| &inner[..])
    }

    pub fn idx_to_lvl(&self, idx: Idx) -> Lvl {
        let fst = self.bound.len() - 1 - idx.fst;
        let snd = self.bound[fst].len() - 1 - idx.snd;
        Lvl { fst, snd }
    }

    pub fn lvl_to_idx(&self, lvl: Lvl) -> Idx {
        let fst = self.bound.len() - 1 - lvl.fst;
        let snd = self.bound[lvl.fst].len() - 1 - lvl.snd;
        Idx { fst, snd }
    }

    pub fn var_to_lvl(&self, var: Var) -> Lvl {
        match var {
            Var::Lvl(lvl) => lvl,
            Var::Idx(idx) => self.idx_to_lvl(idx),
        }
    }
    pub fn var_to_idx(&self, var: Var) -> Idx {
        match var {
            Var::Lvl(lvl) => self.lvl_to_idx(lvl),
            Var::Idx(idx) => idx,
        }
    }
}

impl<T> From<Vec<Vec<T>>> for GenericCtx<T> {
    fn from(value: Vec<Vec<T>>) -> Self {
        GenericCtx { bound: value }
    }
}

/// Defines the interface of a variable context
pub trait Context: Sized {
    /// The type of elements that are stored in the context
    type Elem: Clone;

    /// Get the element bound at `var`
    fn lookup<V: Into<Var> + std::fmt::Debug>(&self, var: V) -> Self::Elem;

    /// Add a new telescope to the context
    /// This function is run if a new list of binders should be pushed to the context
    fn push_telescope(&mut self);

    /// Remove a telescope from the context
    /// Function that is run if the most-recently pushed list of binders should be unbound from the context
    fn pop_telescope(&mut self);

    /// Push a binder into the most-recently pushed telescope
    fn push_binder(&mut self, elem: Self::Elem);

    /// Pop a binder from the most-recently pushed telescope
    fn pop_binder(&mut self, elem: Self::Elem);
}

/// Interface to bind variables to anything that has a `Context`
///
/// There are two use cases for this trait.
///
/// Case 1: You have a type that implements `Context`.
/// Then, a blanket impl ensures that this type also implements `BindContext`.
///
/// Case 2: You have a type that has a field which implements `Context`.
/// Then, only implement the `ctx_mut` method for `BindContext` and return the field that implements `Context`.
/// Do not override the default implementations for the `bind_*` methods.
///
/// In both cases, `BindContext` will provide a safe interface to bind variables and telescopes.
pub trait BindContext: Sized {
    type Ctx: Context;

    fn ctx_mut(&mut self) -> &mut Self::Ctx;

    /// Bind a single element
    fn bind_single<T, O, F>(&mut self, elem: T, f: F) -> O
    where
        T: ContextElem<Self::Ctx>,
        F: FnOnce(&mut Self) -> O,
    {
        self.bind_iter([elem].into_iter(), f)
    }

    /// Bind an iterator `iter` of binders
    fn bind_iter<T, I, O, F>(&mut self, iter: I, f: F) -> O
    where
        T: ContextElem<Self::Ctx>,
        I: Iterator<Item = T>,
        F: FnOnce(&mut Self) -> O,
    {
        self.bind_fold(iter, (), |_ctx, (), _x| (), |ctx, ()| f(ctx))
    }

    /// Bind an iterator `iter` of elements
    ///
    /// Fold the iterator and consume the result
    /// under the inner context with all binders in scope.
    ///
    /// This is used for checking telescopes.
    ///
    /// * `iter` - An iterator of binders
    /// * `acc` - Accumulator for folding the iterator
    /// * `f_acc` - Accumulator function run for each binder
    /// * `f_inner` - Inner function computing the final result under the context of all binders
    fn bind_fold<T, I: Iterator<Item = T>, O1, O2, F1, F2>(
        &mut self,
        iter: I,
        acc: O1,
        f_acc: F1,
        f_inner: F2,
    ) -> O2
    where
        T: ContextElem<Self::Ctx>,
        F1: Fn(&mut Self, O1, T) -> O1,
        F2: FnOnce(&mut Self, O1) -> O2,
    {
        self.bind_fold2(
            iter,
            acc,
            |this, acc, x| BindElem { elem: x.as_element(), ret: f_acc(this, acc, x) },
            f_inner,
        )
    }

    fn bind_fold2<T, I: Iterator<Item = T>, O1, O2, F1, F2>(
        &mut self,
        iter: I,
        acc: O1,
        f_acc: F1,
        f_inner: F2,
    ) -> O2
    where
        F1: Fn(&mut Self, O1, T) -> BindElem<<Self::Ctx as Context>::Elem, O1>,
        F2: FnOnce(&mut Self, O1) -> O2,
    {
        self.bind_fold_failable(
            iter,
            acc,
            |this, acc, x| Result::<_, ()>::Ok(f_acc(this, acc, x)),
            f_inner,
        )
        .unwrap()
    }

    fn bind_fold_failable<T, I: Iterator<Item = T>, O1, O2, F1, F2, E>(
        &mut self,
        iter: I,
        acc: O1,
        f_acc: F1,
        f_inner: F2,
    ) -> Result<O2, E>
    where
        F1: Fn(&mut Self, O1, T) -> Result<BindElem<<Self::Ctx as Context>::Elem, O1>, E>,
        F2: FnOnce(&mut Self, O1) -> O2,
    {
        fn bind_inner<This: BindContext, T, I: Iterator<Item = T>, O1, O2, F1, F2, E>(
            this: &mut This,
            mut iter: I,
            acc: O1,
            f_acc: F1,
            f_inner: F2,
        ) -> Result<O2, E>
        where
            F1: Fn(
                &mut This,
                O1,
                T,
            )
                -> Result<BindElem<<<This as BindContext>::Ctx as Context>::Elem, O1>, E>,
            F2: FnOnce(&mut This, O1) -> O2,
        {
            match iter.next() {
                Some(x) => {
                    let BindElem { elem, ret: acc } = f_acc(this, acc, x)?;
                    this.ctx_mut().push_binder(elem.clone());
                    let res = bind_inner(this, iter, acc, f_acc, f_inner);
                    this.ctx_mut().pop_binder(elem);
                    res
                }
                None => Ok(f_inner(this, acc)),
            }
        }

        self.ctx_mut().push_telescope();
        let res = bind_inner(self, iter, acc, f_acc, f_inner);
        self.ctx_mut().pop_telescope();
        res
    }
}

pub struct BindElem<E, O> {
    pub elem: E,
    pub ret: O,
}

pub trait ContextElem<C: Context> {
    fn as_element(&self) -> C::Elem;
}

impl<C: Context> BindContext for C {
    type Ctx = Self;

    fn ctx_mut(&mut self) -> &mut Self::Ctx {
        self
    }
}
