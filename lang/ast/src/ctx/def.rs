//! Generic definition of variable contexts

use derivative::Derivative;
use pretty::DocAllocator;
use printer::{tokens::*, Alloc, Builder, Print, PrintCfg};

use crate::{Idx, Lvl, Shift, ShiftRange, Var};

use super::values::Binder;

#[derive(Debug, Clone, Default, Derivative)]
#[derivative(Eq, PartialEq, Hash)]
pub struct GenericCtx<T> {
    pub bound: Vec<Vec<Binder<T>>>,
}

impl<T: Clone> GenericCtx<T> {
    pub fn lookup<V: Into<Var>>(&self, idx: V) -> Binder<T> {
        let lvl = self.var_to_lvl(idx.into());
        self.bound
            .get(lvl.fst)
            .and_then(|ctx| ctx.get(lvl.snd))
            .unwrap_or_else(|| panic!("Unbound variable {lvl}"))
            .clone()
    }
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

    pub fn iter(&self) -> impl Iterator<Item = &[Binder<T>]> {
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

impl<T: Shift> GenericCtx<T> {
    fn push_telescope(&mut self) {
        self.shift(&(0..), (1, 0));
        self.bound.push(vec![]);
    }

    fn pop_telescope(&mut self) {
        self.bound.pop().unwrap();
        self.shift(&(0..), (-1, 0));
    }

    fn push_binder(&mut self, elem: Binder<T>) {
        self.bound.last_mut().expect("Cannot push without calling level_inc_fst first").push(elem);
        self.shift_at_lvl(&(0..1), self.bound.len() - 1, (0, 1));
    }

    fn pop_binder(&mut self, _elem: Binder<T>) {
        let err = "Cannot pop from empty context";
        self.bound.last_mut().expect(err).pop().expect(err);
        self.shift_at_lvl(&(0..1), self.bound.len() - 1, (0, -1));
    }

    pub fn shift<R: ShiftRange>(&mut self, range: &R, by: (isize, isize)) {
        for lvl in 0..self.bound.len() {
            self.shift_at_lvl(range, lvl, by)
        }
    }

    fn shift_at_lvl<R: ShiftRange>(&mut self, range: &R, lvl: usize, by: (isize, isize)) {
        for i in 0..self.bound[lvl].len() {
            self.bound[lvl][i].shift_in_range(range, by);
        }
    }
}

impl<T> From<Vec<Vec<Binder<T>>>> for GenericCtx<T> {
    fn from(value: Vec<Vec<Binder<T>>>) -> Self {
        GenericCtx { bound: value }
    }
}

impl<T: Print> Print for GenericCtx<T> {
    fn print<'a>(&'a self, cfg: &PrintCfg, alloc: &'a Alloc<'a>) -> Builder<'a> {
        let sep = alloc.text(COMMA).append(alloc.space());
        let iter = self.iter().map(|ctx| {
            alloc
                .intersperse(
                    ctx.iter().map(|b| {
                        b.name
                            .print(cfg, alloc)
                            .append(COLON)
                            .append(alloc.space())
                            .append(b.content.print(cfg, alloc))
                    }),
                    sep.clone(),
                )
                .brackets()
        });
        alloc.intersperse(iter, sep.clone()).brackets()
    }
}

/// Interface to bind variables to anything that has a `GenericCtx`.
///
/// There are two ways to use this trait.
///
/// Case 1: It is implemented for `GenericCtx`.
///
/// Case 2: You have a type that has a field of type `GenericCtx`.
/// Then, only implement the `ctx_mut` method for `BindContext` and return the field that implements `Context`.
/// Do not override the default implementations for the `bind_*` methods.
///
/// In both cases, `BindContext` will provide a safe interface to bind variables and telescopes.
pub trait BindContext: Sized {
    type Content: Shift + Clone;

    /// Get a mutable reference to the context
    fn ctx_mut(&mut self) -> &mut GenericCtx<Self::Content>;

    /// Bind a single binder as a one-element telescope
    fn bind_single<T, O, F>(&mut self, elem: T, f: F) -> O
    where
        T: AsBinder<Self::Content>,
        F: FnOnce(&mut Self) -> O,
    {
        self.bind_iter([elem].into_iter(), f)
    }

    /// Bind an iterator `iter` of binders
    fn bind_iter<T, I, O, F>(&mut self, iter: I, f: F) -> O
    where
        T: AsBinder<Self::Content>,
        I: Iterator<Item = T>,
        F: FnOnce(&mut Self) -> O,
    {
        self.bind_fold(iter, (), |_ctx, (), x| x.as_binder(), |ctx, ()| f(ctx))
    }

    /// Bind a telescope, binder by binder and simultaneously fold over the binders
    ///
    /// Parameters:
    /// * `iter`: An iterator of elements to bind
    /// * `acc`: Initial fold accumulator
    /// * `f_acc`: Function called on each element in `iter`
    ///            The return value is the binder to bind.
    ///            The function can mutate the accumulator.
    /// * `f_inner`: Function called on a context where all binders have been bound
    fn bind_fold<T, I: Iterator<Item = T>, O1, O2, F1, F2>(
        &mut self,
        iter: I,
        acc: O1,
        f_acc: F1,
        f_inner: F2,
    ) -> O2
    where
        F1: Fn(&mut Self, &mut O1, T) -> Binder<Self::Content>,
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

    /// Like `bind_fold`, but allows the accumulator function `f_acc` to fail
    fn bind_fold_failable<T, I: Iterator<Item = T>, O1, O2, F1, F2, E>(
        &mut self,
        iter: I,
        acc: O1,
        f_acc: F1,
        f_inner: F2,
    ) -> Result<O2, E>
    where
        F1: Fn(&mut Self, &mut O1, T) -> Result<Binder<Self::Content>, E>,
        F2: FnOnce(&mut Self, O1) -> O2,
    {
        fn bind_inner<This: BindContext, T, I: Iterator<Item = T>, O1, O2, F1, F2, E>(
            this: &mut This,
            mut iter: I,
            mut acc: O1,
            f_acc: F1,
            f_inner: F2,
        ) -> Result<O2, E>
        where
            F1: Fn(&mut This, &mut O1, T) -> Result<Binder<<This as BindContext>::Content>, E>,
            F2: FnOnce(&mut This, O1) -> O2,
        {
            match iter.next() {
                Some(x) => {
                    let elem = f_acc(this, &mut acc, x)?;
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

pub trait AsBinder<T> {
    fn as_binder(&self) -> Binder<T>;
}

impl<T: Shift + Clone> BindContext for GenericCtx<T> {
    type Content = T;

    fn ctx_mut(&mut self) -> &mut GenericCtx<Self::Content> {
        self
    }
}

impl<T: Clone> AsBinder<T> for Binder<T> {
    fn as_binder(&self) -> Binder<T> {
        self.clone()
    }
}
