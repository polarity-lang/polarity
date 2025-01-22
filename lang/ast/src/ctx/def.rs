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

impl<T: Shift + Clone> Context for GenericCtx<T> {
    type Elem = Binder<T>;

    fn lookup<V: Into<Var>>(&self, idx: V) -> Self::Elem {
        let lvl = self.var_to_lvl(idx.into());
        self.bound
            .get(lvl.fst)
            .and_then(|ctx| ctx.get(lvl.snd))
            .unwrap_or_else(|| panic!("Unbound variable {lvl}"))
            .clone()
    }

    fn push_telescope(&mut self) {
        self.shift(&(0..), (1, 0));
        self.bound.push(vec![]);
    }

    fn pop_telescope(&mut self) {
        self.bound.pop().unwrap();
        self.shift(&(0..), (-1, 0));
    }

    fn push_binder(&mut self, elem: Self::Elem) {
        self.bound.last_mut().expect("Cannot push without calling level_inc_fst first").push(elem);
        self.shift_at_lvl(&(0..1), self.bound.len() - 1, (0, 1));
    }

    fn pop_binder(&mut self, _elem: Self::Elem) {
        let err = "Cannot pop from empty context";
        self.bound.last_mut().expect(err).pop().expect(err);
        self.shift_at_lvl(&(0..1), self.bound.len() - 1, (0, -1));
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
/// There are two ways to use this trait.
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
        {
            self.bind_fold2(
                iter,
                (),
                |_ctx, (), x| BindElem { elem: x.as_element(), ret: () },
                |ctx, ()| f(ctx),
            )
        }
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

impl<C: Context> ContextElem<C> for C::Elem {
    fn as_element(&self) -> C::Elem {
        self.clone()
    }
}
