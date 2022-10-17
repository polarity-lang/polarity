use syntax::common::*;
use syntax::de_bruijn::*;
use syntax::named::Named;

#[derive(Debug, Clone)]
pub struct Ctx {
    bound: Vec<Vec<Ident>>,
}

impl Ctx {
    pub fn empty() -> Self {
        Self { bound: vec![] }
    }

    pub fn bound(&self, idx: Idx) -> Ident {
        // FIXME: Handle shadowing
        self.bound
            .get(self.bound.len() - 1 - idx.fst)
            .and_then(|ctx| ctx.get(ctx.len() - 1 - idx.snd))
            .expect("Unbound variable")
            .clone()
    }

    /// Bind a single name
    pub fn bind<T, F>(&mut self, name: Ident, f: F) -> T
    where
        F: Fn(&mut Ctx) -> T,
    {
        self.bind_fold([name].into_iter(), (), |_, _, _| (), |ctx, _| f(ctx))
    }

    /// Bind an iterator `iter` of binders
    ///
    /// Fold the iterator and consume the result
    /// under the inner context with all binders in scope.
    ///
    /// * `iter` - An iterator of binders implementing `Named`.
    /// * `acc` - Accumulator for folding the iterator
    /// * `f_acc` - Accumulator function run for each binder
    /// * `f_inner` - Inner function computing the final result under the context of all binders
    pub fn bind_fold<T, I: Iterator<Item = T>, O1, O2, F1, F2>(
        &mut self,
        iter: I,
        acc: O1,
        f_acc: F1,
        f_inner: F2,
    ) -> O2
    where
        T: Named,
        F1: Fn(&mut Ctx, O1, T) -> O1,
        F2: FnOnce(&mut Ctx, O1) -> O2,
    {
        fn bind_inner<T, I: Iterator<Item = T>, O1, O2, F1, F2>(
            ctx: &mut Ctx,
            mut iter: I,
            acc: O1,
            f_acc: F1,
            f_inner: F2,
        ) -> O2
        where
            T: Named,
            F1: Fn(&mut Ctx, O1, T) -> O1,
            F2: FnOnce(&mut Ctx, O1) -> O2,
        {
            match iter.next() {
                Some(x) => {
                    let name = x.name().clone();
                    let acc = f_acc(ctx, acc, x);
                    ctx.push(name);
                    let res = bind_inner(ctx, iter, acc, f_acc, f_inner);
                    ctx.pop();
                    res
                }
                None => f_inner(ctx, acc),
            }
        }

        self.level_inc_fst();
        let res = bind_inner(self, iter, acc, f_acc, f_inner);
        self.level_dec_fst();
        res
    }

    /// Increment the first component of the current De-Bruijn level
    fn level_inc_fst(&mut self) {
        self.bound.push(vec![]);
    }

    /// Decrement the first component of the current De-Bruijn level
    fn level_dec_fst(&mut self) {
        self.bound.pop().unwrap();
    }

    /// Push a binder contained in a binder list, incrementing the second dimension of the current De Bruijn level
    fn push(&mut self, name: Ident) {
        self.bound.last_mut().expect("Cannot push without calling level_inc_fst first").push(name);
    }

    /// Push a binder contained in a binder list, decrementing the second dimension of the current De Bruijn level
    fn pop(&mut self) {
        let err = "Cannot pop from empty context";
        self.bound.last_mut().expect(err).pop().expect(err);
    }
}
