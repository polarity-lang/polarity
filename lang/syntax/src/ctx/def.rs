//! Generic definition of variable contexts

use std::rc::Rc;

use crate::ast::Annotated;
use crate::ast::*;
use crate::common::*;
use crate::named::Named;

/// Defines the interface of a variable context
pub trait Context: Sized {
    /// The type of elements that can be bound to the context
    type ElemIn;
    /// Result type of a lookup into the context
    type ElemOut;
    /// Variable type that can be looked up in the context
    type Var;

    /// Create an empty context
    fn empty() -> Self;

    /// Get the element bound at `var`
    fn lookup<V: Into<Self::Var>>(&self, var: V) -> Self::ElemOut;

    /// Add a new telescope to the context
    /// This function is run if a new list of binders should be pushed to the context
    fn push_telescope(&mut self);

    /// Remove a telescope from the context
    /// Function that is run if the most-recently pushed list of binders should be unbound from the context
    fn pop_telescope(&mut self);

    /// Push a binder into the most-recently pushed telescope
    fn push_binder(&mut self, elem: Self::ElemIn);

    /// Pop a binder from the most-recently pushed telescope
    fn pop_binder(&mut self, elem: Self::ElemIn);
}

pub trait HasContext {
    type Ctx: Context;

    fn ctx_mut(&mut self) -> &mut Self::Ctx;
}

pub trait Bind {
    type Elem;

    /// Bind a single element
    fn bind_single<T, O, F>(&mut self, elem: T, f: F) -> O
    where
        T: AsElement<Self::Elem>,
        F: Fn(&mut Self) -> O,
    {
        self.bind_iter([elem].into_iter(), f)
    }

    /// Bind an iterator `iter` of binders
    fn bind_iter<T, I, O, F>(&mut self, iter: I, f: F) -> O
    where
        T: AsElement<Self::Elem>,
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
        T: AsElement<Self::Elem>,
        F1: Fn(&mut Self, O1, T) -> O1,
        F2: FnOnce(&mut Self, O1) -> O2;
}

impl<C: Context> HasContext for C {
    type Ctx = Self;

    fn ctx_mut(&mut self) -> &mut Self {
        self
    }
}

impl<C: HasContext> Bind for C {
    type Elem = <<Self as HasContext>::Ctx as Context>::ElemIn;

    fn bind_fold<T, I: Iterator<Item = T>, O1, O2, F1, F2>(
        &mut self,
        iter: I,
        acc: O1,
        f_acc: F1,
        f_inner: F2,
    ) -> O2
    where
        T: AsElement<Self::Elem>,
        F1: Fn(&mut Self, O1, T) -> O1,
        F2: FnOnce(&mut Self, O1) -> O2,
    {
        fn bind_inner<This: HasContext, T, I: Iterator<Item = T>, O1, O2, F1, F2>(
            this: &mut This,
            mut iter: I,
            acc: O1,
            f_acc: F1,
            f_inner: F2,
        ) -> O2
        where
            T: AsElement<<<This as HasContext>::Ctx as Context>::ElemIn>,
            F1: Fn(&mut This, O1, T) -> O1,
            F2: FnOnce(&mut This, O1) -> O2,
        {
            match iter.next() {
                Some(x) => {
                    let elem1 = x.as_element();
                    let elem2 = x.as_element();
                    let acc = f_acc(this, acc, x);
                    this.ctx_mut().push_binder(elem1);
                    let res = bind_inner(this, iter, acc, f_acc, f_inner);
                    this.ctx_mut().pop_binder(elem2);
                    res
                }
                None => f_inner(this, acc),
            }
        }

        self.ctx_mut().push_telescope();
        let res = bind_inner(self, iter, acc, f_acc, f_inner);
        self.ctx_mut().pop_telescope();
        res
    }
}

pub trait AsElement<E> {
    fn as_element(&self) -> E;
}

impl<T> AsElement<()> for T {
    fn as_element(&self) {}
}

impl<P: Phase, T: Annotated<P>> AsElement<Rc<Exp<P>>> for T {
    fn as_element(&self) -> Rc<Exp<P>> {
        self.typ()
    }
}

impl<T: Named> AsElement<Ident> for T {
    fn as_element(&self) -> Ident {
        self.name().to_owned()
    }
}
