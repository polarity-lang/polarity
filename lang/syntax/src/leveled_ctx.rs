use crate::ast::Swap;

use super::de_bruijn::*;

#[derive(Clone, Debug)]
pub struct LeveledCtx {
    /// Number of binders in the second dimension for each first dimension
    bound: Vec<usize>,
}

impl LeveledCtx {
    pub fn empty() -> Self {
        Self { bound: vec![] }
    }

    /// Bind an iterator `iter` of binders
    pub fn bind<T, I, O, F>(&mut self, iter: I, f: F) -> O
    where
        I: Iterator<Item = T>,
        F: FnOnce(&mut LeveledCtx) -> O,
    {
        self.bind_fold(iter, (), |_ctx, (), _x| (), |ctx, ()| f(ctx))
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
        F1: Fn(&mut LeveledCtx, O1, T) -> O1,
        F2: FnOnce(&mut LeveledCtx, O1) -> O2,
    {
        fn bind_inner<T, I: Iterator<Item = T>, O1, O2, F1, F2>(
            ctx: &mut LeveledCtx,
            mut iter: I,
            acc: O1,
            f_acc: F1,
            f_inner: F2,
        ) -> O2
        where
            F1: Fn(&mut LeveledCtx, O1, T) -> O1,
            F2: FnOnce(&mut LeveledCtx, O1) -> O2,
        {
            match iter.next() {
                Some(x) => {
                    let acc = f_acc(ctx, acc, x);
                    ctx.push();
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
        self.bound.push(0);
    }

    /// Decrement the first component of the current De-Bruijn level
    fn level_dec_fst(&mut self) {
        self.bound.pop().unwrap();
    }

    /// Push a binder contained in a binder list, incrementing the second dimension of the current De Bruijn level
    fn push(&mut self) {
        *self.bound.last_mut().expect("Cannot push without calling level_inc_fst first") += 1;
    }

    /// Push a binder contained in a binder list, decrementing the second dimension of the current De Bruijn level
    fn pop(&mut self) {
        let err = "Cannot pop from empty context";
        *self.bound.last_mut().expect(err) -= 1;
    }
}

impl Leveled for LeveledCtx {
    fn idx_to_lvl(&self, idx: Idx) -> Lvl {
        let fst = self.bound.len() - 1 - idx.fst;
        let snd = self.bound[fst] - 1 - idx.snd;
        Lvl { fst, snd }
    }

    fn lvl_to_idx(&self, lvl: Lvl) -> Idx {
        let fst = self.bound.len() - 1 - lvl.fst;
        let snd = self.bound[lvl.fst] - 1 - lvl.snd;
        Idx { fst, snd }
    }
}

impl Swap for LeveledCtx {
    fn swap(&self, fst1: usize, fst2: usize) -> Self {
        let mut new_ctx = self.clone();
        new_ctx.bound.swap(fst1, fst2);
        new_ctx
    }
}
