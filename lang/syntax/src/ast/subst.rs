use std::rc::Rc;

use crate::de_bruijn::*;

use super::untyped::*;

pub struct Assign<K, V>(pub K, pub V);

pub trait Substitution {
    fn get(&self, lvl: Lvl) -> Option<Rc<Exp>>;
}

impl<T: AsRef<[Rc<Exp>]>> Substitution for T {
    fn get(&self, lvl: Lvl) -> Option<Rc<Exp>> {
        if lvl.fst != 0 {
            return None;
        }
        Some(self.as_ref()[lvl.snd].clone())
    }
}

impl Substitution for Assign<Lvl, &Rc<Exp>> {
    fn get(&self, lvl: Lvl) -> Option<Rc<Exp>> {
        if self.0 == lvl {
            Some(self.1.clone())
        } else {
            None
        }
    }
}

pub trait Substitutable: Sized {
    fn subst<L: Leveled, S: Substitution>(&self, ctx: &L, by: &S) -> Self;
}

pub trait SubstTelescope {
    fn subst_telescope<S: Substitution>(&self, lvl: Lvl, by: &S) -> Self;
}

// Swap the given indices
pub trait Swap {
    fn swap(&self, fst1: usize, fst2: usize) -> Self;
}

pub trait SwapWithCtx {
    fn swap_with_ctx<L: Leveled + Swap>(&self, ctx: &L, fst1: usize, fst2: usize) -> Self;
}

impl Substitutable for Rc<Exp> {
    fn subst<L: Leveled, S: Substitution>(&self, ctx: &L, by: &S) -> Self {
        match &**self {
            Exp::Var { info, name, idx } => match by.get(ctx.idx_to_lvl(*idx)) {
                Some(exp) => exp,
                None => Rc::new(Exp::Var { info: info.clone(), name: name.clone(), idx: *idx }),
            },
            Exp::TypCtor { info, name, args } => Rc::new(Exp::TypCtor {
                info: info.clone(),
                name: name.clone(),
                args: args.subst(ctx, by),
            }),
            Exp::Ctor { info, name, args } => Rc::new(Exp::Ctor {
                info: info.clone(),
                name: name.clone(),
                args: args.subst(ctx, by),
            }),
            Exp::Dtor { info, exp, name, args } => Rc::new(Exp::Dtor {
                info: info.clone(),
                exp: exp.subst(ctx, by),
                name: name.clone(),
                args: args.subst(ctx, by),
            }),
            Exp::Anno { info, exp, typ } => Rc::new(Exp::Anno {
                info: info.clone(),
                exp: exp.subst(ctx, by),
                typ: typ.subst(ctx, by),
            }),
            Exp::Type { info } => Rc::new(Exp::Type { info: info.clone() }),
        }
    }
}

impl Substitutable for TypApp {
    fn subst<L: Leveled, S: Substitution>(&self, ctx: &L, by: &S) -> Self {
        let TypApp { info, name, args: subst } = self;
        TypApp { info: info.clone(), name: name.clone(), args: subst.subst(ctx, by) }
    }
}

impl Substitutable for Args {
    fn subst<L: Leveled, S: Substitution>(&self, ctx: &L, by: &S) -> Self {
        self.iter().map(|x| x.subst(ctx, by)).collect()
    }
}

impl Substitutable for Param {
    fn subst<L: Leveled, S: Substitution>(&self, ctx: &L, by: &S) -> Self {
        let Param { name, typ } = self;
        Param { name: name.clone(), typ: typ.subst(ctx, by) }
    }
}

impl<T: Substitutable> SwapWithCtx for T {
    fn swap_with_ctx<L: Leveled + Swap>(&self, ctx: &L, fst1: usize, fst2: usize) -> Self {
        self.subst(ctx, &SwapSubst { fst1, fst2, ctx })
    }
}

struct SwapSubst<'a, L: Leveled + Swap> {
    fst1: usize,
    fst2: usize,
    // FIXME: With local bindings, this will have to be context-dependent
    ctx: &'a L,
}

impl<'a, L: Leveled + Swap> Substitution for SwapSubst<'a, L> {
    fn get(&self, lvl: Lvl) -> Option<Rc<Exp>> {
        let new_lvl = if lvl.fst == self.fst1 {
            Some(Lvl { fst: self.fst2, snd: lvl.snd })
        } else if lvl.fst == self.fst2 {
            Some(Lvl { fst: self.fst1, snd: lvl.snd })
        } else {
            None
        };

        let new_ctx = self.ctx.swap(self.fst1, self.fst2);

        new_lvl.map(|new_lvl| {
            Rc::new(Exp::Var {
                info: Info::empty(),
                name: "".to_owned(),
                idx: new_ctx.lvl_to_idx(new_lvl),
            })
        })
    }
}
