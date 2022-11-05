use std::rc::Rc;

use crate::de_bruijn::*;
use crate::leveled_ctx::LeveledCtx;

use super::untyped::*;

pub struct Assign<K, V>(pub K, pub V);

pub type Ctx = LeveledCtx;

pub trait Substitution {
    fn get(&self, ctx: &Ctx, lvl: Lvl) -> Option<Rc<Exp>>;
}

impl<T: AsRef<[Rc<Exp>]>> Substitution for T {
    fn get(&self, _ctx: &Ctx, lvl: Lvl) -> Option<Rc<Exp>> {
        if lvl.fst != 0 {
            return None;
        }
        Some(self.as_ref()[lvl.snd].clone())
    }
}

impl Substitution for Assign<Lvl, &Rc<Exp>> {
    fn get(&self, _ctx: &Ctx, lvl: Lvl) -> Option<Rc<Exp>> {
        if self.0 == lvl {
            Some(self.1.clone())
        } else {
            None
        }
    }
}

pub trait Substitutable: Sized {
    fn subst<S: Substitution>(&self, ctx: &mut Ctx, by: &S) -> Self;
}

pub trait SubstTelescope {
    fn subst_telescope<S: Substitution>(&self, lvl: Lvl, by: &S) -> Self;
}

// Swap the given indices
pub trait Swap {
    fn swap(&self, fst1: usize, fst2: usize) -> Self;
}

pub trait SwapWithCtx {
    fn swap_with_ctx(&self, ctx: &mut Ctx, fst1: usize, fst2: usize) -> Self;
}

impl Substitutable for Rc<Exp> {
    fn subst<S: Substitution>(&self, ctx: &mut Ctx, by: &S) -> Self {
        match &**self {
            Exp::Var { info, name, idx } => match by.get(ctx, ctx.idx_to_lvl(*idx)) {
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
            Exp::Match { info, name, on_exp, body } => Rc::new(Exp::Match {
                info: info.clone(),
                name: name.clone(),
                on_exp: on_exp.subst(ctx, by),
                body: body.subst(ctx, by),
            }),
            Exp::Comatch { info, name, body } => Rc::new(Exp::Comatch {
                info: info.clone(),
                name: name.clone(),
                body: body.subst(ctx, by),
            }),
        }
    }
}

impl Substitutable for Match {
    fn subst<S: Substitution>(&self, ctx: &mut Ctx, by: &S) -> Self {
        let Match { info, cases } = self;
        Match { info: info.clone(), cases: cases.iter().map(|case| case.subst(ctx, by)).collect() }
    }
}

impl Substitutable for Comatch {
    fn subst<S: Substitution>(&self, ctx: &mut Ctx, by: &S) -> Self {
        let Comatch { info, cases } = self;
        Comatch {
            info: info.clone(),
            cases: cases.iter().map(|cocase| cocase.subst(ctx, by)).collect(),
        }
    }
}

impl Substitutable for Case {
    fn subst<S: Substitution>(&self, ctx: &mut Ctx, by: &S) -> Self {
        let Case { info, name, args, body } = self;
        ctx.bind(args.params.iter(), |ctx| Case {
            info: info.clone(),
            name: name.clone(),
            args: args.clone(),
            body: body.as_ref().map(|body| body.subst(ctx, by)),
        })
    }
}

impl Substitutable for Cocase {
    fn subst<S: Substitution>(&self, ctx: &mut Ctx, by: &S) -> Self {
        let Cocase { info, name, args, body } = self;
        ctx.bind(args.params.iter(), |ctx| Cocase {
            info: info.clone(),
            name: name.clone(),
            args: args.clone(),
            body: body.as_ref().map(|body| body.subst(ctx, by)),
        })
    }
}

impl Substitutable for TypApp {
    fn subst<S: Substitution>(&self, ctx: &mut Ctx, by: &S) -> Self {
        let TypApp { info, name, args: subst } = self;
        TypApp { info: info.clone(), name: name.clone(), args: subst.subst(ctx, by) }
    }
}

impl Substitutable for Args {
    fn subst<S: Substitution>(&self, ctx: &mut Ctx, by: &S) -> Self {
        self.iter().map(|x| x.subst(ctx, by)).collect()
    }
}

impl Substitutable for Param {
    fn subst<S: Substitution>(&self, ctx: &mut Ctx, by: &S) -> Self {
        let Param { name, typ } = self;
        Param { name: name.clone(), typ: typ.subst(ctx, by) }
    }
}

impl<T: Substitutable> SwapWithCtx for T {
    fn swap_with_ctx(&self, ctx: &mut Ctx, fst1: usize, fst2: usize) -> Self {
        self.subst(ctx, &SwapSubst { fst1, fst2 })
    }
}

struct SwapSubst {
    fst1: usize,
    fst2: usize,
}

impl Substitution for SwapSubst {
    fn get(&self, ctx: &Ctx, lvl: Lvl) -> Option<Rc<Exp>> {
        let new_lvl = if lvl.fst == self.fst1 {
            Some(Lvl { fst: self.fst2, snd: lvl.snd })
        } else if lvl.fst == self.fst2 {
            Some(Lvl { fst: self.fst1, snd: lvl.snd })
        } else {
            None
        };

        let new_ctx = ctx.swap(self.fst1, self.fst2);

        new_lvl.map(|new_lvl| {
            Rc::new(Exp::Var {
                info: Info::empty(),
                name: "".to_owned(),
                idx: new_ctx.lvl_to_idx(new_lvl),
            })
        })
    }
}
