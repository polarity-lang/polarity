use std::collections::HashMap;

use syntax::ast;
use syntax::common::*;
use syntax::cst;
use syntax::de_bruijn::*;
use syntax::named::Named;

use super::result::LoweringError;

pub struct Ctx {
    /// For each name, store a vector representing the different binders
    /// represented by this name. The last entry represents the binder currently in scope,
    /// the remaining entries represent the binders which are currently shadowed.
    ///
    /// Bound variables in this map are De-Bruijn levels rather than indices:
    map: HashMap<Ident, Vec<Elem>>,
    /// Accumulates top-level declarations
    decls: ast::Decls,
    /// Counts the number of entries for each De-Bruijn level
    levels: Vec<usize>,
}

impl Ctx {
    pub fn empty() -> Self {
        Self { map: HashMap::new(), decls: ast::Decls::empty(), levels: Vec::new() }
    }

    pub fn lookup(&self, name: &Ident) -> Result<&Elem, LoweringError> {
        self.map
            .get(name)
            .and_then(|stack| stack.last())
            .ok_or_else(|| LoweringError::UndefinedIdent(name.clone()))
    }

    pub fn lower_bound(&self, lvl: Lvl) -> Idx {
        self.level_to_index(lvl)
    }

    pub fn add_name(&mut self, name: &Ident, decl_kind: DeclKind) -> Result<(), LoweringError> {
        let var = Elem::Decl(decl_kind);
        let stack = self.map.entry(name.clone()).or_insert_with(Default::default);
        stack.push(var);
        Ok(())
    }

    pub fn add_decls<I>(&mut self, decls: I) -> Result<(), LoweringError>
    where
        I: IntoIterator<Item = ast::Decl>,
    {
        decls.into_iter().try_for_each(|decl| self.add_decl(decl))
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

    pub fn into_decls(self) -> ast::Decls {
        self.decls
    }

    /// Bind a single name
    pub fn bind<T, F: Fn(&mut Ctx) -> T>(&mut self, name: Ident, f: F) -> T {
        self.bind_fold([name].iter(), (), |_, _, _| (), |ctx, _| f(ctx))
    }

    /// Bind an iterator `iter` of `Named` binders.
    ///
    /// Fold the iterator and consume the result
    /// under the inner context with all binders in scope.
    ///
    /// This is used for lowering telescopes.
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
        F2: Fn(&mut Ctx, O1) -> O2,
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
            F2: Fn(&mut Ctx, O1) -> O2,
        {
            match iter.next() {
                Some(x) => {
                    let name = x.name().clone();
                    ctx.push_idx(name.clone());
                    let acc = f_acc(ctx, acc, x);
                    let res = bind_inner(ctx, iter, acc, f_acc, f_inner);
                    ctx.pop_idx(&name);
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

    /// Push a binder contained in a binder list, incrementing the second dimension of the current De Bruijn level
    fn push_idx(&mut self, name: Ident) {
        let var = Elem::Bound(self.curr_lvl());
        self.level_inc_snd();
        let stack = self.map.entry(name).or_insert_with(Default::default);
        stack.push(var);
    }

    /// Push a binder contained in a binder list, decrementing the first dimension of the current De Bruijn level
    fn pop_idx(&mut self, name: &Ident) {
        let stack = self.map.get_mut(name).expect("Tried to read unknown variable");
        stack.pop().unwrap();
        self.level_dec_snd();
    }

    /// Next De Bruijn level to be assigned
    fn curr_lvl(&self) -> Lvl {
        let fst = self.levels.len() - 1;
        let snd = *self.levels.last().unwrap_or(&0);
        Lvl { fst, snd }
    }

    /// Convert the given De-Bruijn level to a De-Bruijn index
    fn level_to_index(&self, lvl: Lvl) -> Idx {
        let fst = self.levels.len() - 1 - lvl.fst;
        let snd = self.levels[lvl.fst] - 1 - lvl.snd;
        Idx { fst, snd }
    }

    /// Increment the first component of the current De-Bruijn level
    fn level_inc_fst(&mut self) {
        self.levels.push(0);
    }

    /// Decrement the first component of the current De-Bruijn level
    fn level_dec_fst(&mut self) {
        self.levels.pop().unwrap();
    }

    /// Increment the second component of the current De-Bruijn level
    fn level_inc_snd(&mut self) {
        *self.levels.last_mut().unwrap() += 1;
    }

    /// Decrement the second component of the current De-Bruijn level
    fn level_dec_snd(&mut self) {
        *self.levels.last_mut().unwrap() -= 1;
    }
}

#[derive(Clone, Debug)]
pub enum Elem {
    Bound(Lvl),
    Decl(DeclKind),
}

#[derive(Clone, Debug)]
pub enum DeclKind {
    Data,
    Codata,
    Def,
    Codef,
    Ctor,
    Dtor,
}

impl From<&cst::Decl> for DeclKind {
    fn from(decl: &cst::Decl) -> Self {
        match decl {
            cst::Decl::Data(_) => Self::Data,
            cst::Decl::Codata(_) => Self::Codata,
            cst::Decl::Def(_) => Self::Def,
            cst::Decl::Codef(_) => Self::Codef,
        }
    }
}
