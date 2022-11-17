use std::rc::Rc;

use crate::common::*;
use crate::ctx::*;
use crate::de_bruijn::*;
use crate::tst;

use super::generic::*;

pub struct Assign<K, V>(pub K, pub V);

pub type Ctx = LevelCtx;

pub trait Substitution<P: Phase> {
    fn get_subst(&self, ctx: &Ctx, lvl: Lvl) -> Option<Rc<Exp<P>>>;
}

impl<P: Phase, T: AsRef<[Rc<Exp<P>>]>> Substitution<P> for T {
    fn get_subst(&self, _ctx: &Ctx, lvl: Lvl) -> Option<Rc<Exp<P>>> {
        if lvl.fst != 0 {
            return None;
        }
        Some(self.as_ref()[lvl.snd].clone())
    }
}

impl<P: Phase> Substitution<P> for Assign<Lvl, &Rc<Exp<P>>> {
    fn get_subst(&self, _ctx: &Ctx, lvl: Lvl) -> Option<Rc<Exp<P>>> {
        if self.0 == lvl {
            Some(self.1.clone())
        } else {
            None
        }
    }
}

pub trait Substitutable<P: Phase>: Sized {
    fn subst<S: Substitution<P>>(&self, ctx: &mut Ctx, by: &S) -> Self;
}

pub trait SubstTelescope<P: Phase> {
    fn subst_telescope<S: Substitution<P>>(&self, lvl: Lvl, by: &S) -> Self;
}

// Swap the given indices
pub trait Swap {
    fn swap(&self, fst1: usize, fst2: usize) -> Self;
}

pub trait SwapWithCtx<P: Phase> {
    fn swap_with_ctx(&self, ctx: &mut Ctx, fst1: usize, fst2: usize) -> Self;
}

impl<P: Phase> Substitutable<P> for Rc<Exp<P>>
where
    P::Typ: Substitutable<P>,
{
    fn subst<S: Substitution<P>>(&self, ctx: &mut Ctx, by: &S) -> Self {
        match &**self {
            Exp::Var { info, name, idx } => match by.get_subst(ctx, ctx.idx_to_lvl(*idx)) {
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
            Exp::Match { info, name, on_exp, in_typ, body } => Rc::new(Exp::Match {
                info: info.clone(),
                name: name.clone(),
                on_exp: on_exp.subst(ctx, by),
                in_typ: in_typ.subst(ctx, by),
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

impl<P: Phase> Substitutable<P> for Match<P>
where
    P::Typ: Substitutable<P>,
{
    fn subst<S: Substitution<P>>(&self, ctx: &mut Ctx, by: &S) -> Self {
        let Match { info, cases } = self;
        Match { info: info.clone(), cases: cases.iter().map(|case| case.subst(ctx, by)).collect() }
    }
}

impl<P: Phase> Substitutable<P> for Comatch<P>
where
    P::Typ: Substitutable<P>,
{
    fn subst<S: Substitution<P>>(&self, ctx: &mut Ctx, by: &S) -> Self {
        let Comatch { info, cases } = self;
        Comatch {
            info: info.clone(),
            cases: cases.iter().map(|cocase| cocase.subst(ctx, by)).collect(),
        }
    }
}

impl<P: Phase> Substitutable<P> for Case<P>
where
    P::Typ: Substitutable<P>,
{
    fn subst<S: Substitution<P>>(&self, ctx: &mut Ctx, by: &S) -> Self {
        let Case { info, name, args, body } = self;
        ctx.bind_iter(args.params.iter(), |ctx| Case {
            info: info.clone(),
            name: name.clone(),
            args: args.clone(),
            body: body.as_ref().map(|body| body.subst(ctx, by)),
        })
    }
}

impl<P: Phase> Substitutable<P> for Cocase<P>
where
    P::Typ: Substitutable<P>,
{
    fn subst<S: Substitution<P>>(&self, ctx: &mut Ctx, by: &S) -> Self {
        let Cocase { info, name, args, body } = self;
        ctx.bind_iter(args.params.iter(), |ctx| Cocase {
            info: info.clone(),
            name: name.clone(),
            args: args.clone(),
            body: body.as_ref().map(|body| body.subst(ctx, by)),
        })
    }
}

impl<P: Phase> Substitutable<P> for TypApp<P>
where
    P::Typ: Substitutable<P>,
{
    fn subst<S: Substitution<P>>(&self, ctx: &mut Ctx, by: &S) -> Self {
        let TypApp { info, name, args: subst } = self;
        TypApp { info: info.clone(), name: name.clone(), args: subst.subst(ctx, by) }
    }
}

impl<P: Phase> Substitutable<P> for Args<P>
where
    P::Typ: Substitutable<P>,
{
    fn subst<S: Substitution<P>>(&self, ctx: &mut Ctx, by: &S) -> Self {
        self.iter().map(|x| x.subst(ctx, by)).collect()
    }
}

impl<P: Phase> Substitutable<P> for Param<P>
where
    P::Typ: Substitutable<P>,
{
    fn subst<S: Substitution<P>>(&self, ctx: &mut Ctx, by: &S) -> Self {
        let Param { name, typ } = self;
        Param { name: name.clone(), typ: typ.subst(ctx, by) }
    }
}

impl<P: Phase<VarName = Ident>, T: Substitutable<P>> SwapWithCtx<P> for T
where
    P::TypeInfo: Default,
{
    fn swap_with_ctx(&self, ctx: &mut Ctx, fst1: usize, fst2: usize) -> Self {
        self.subst(ctx, &SwapSubst { fst1, fst2 })
    }
}

struct SwapSubst {
    fst1: usize,
    fst2: usize,
}

impl<P: Phase<VarName = Ident>> Substitution<P> for SwapSubst
where
    P::TypeInfo: Default,
{
    fn get_subst(&self, ctx: &Ctx, lvl: Lvl) -> Option<Rc<Exp<P>>> {
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
                info: Default::default(),
                name: "".to_owned(),
                idx: new_ctx.lvl_to_idx(new_lvl),
            })
        })
    }
}

impl<P: Phase> Substitutable<P> for () {
    fn subst<S: Substitution<P>>(&self, _ctx: &mut Ctx, _by: &S) -> Self {}
}

impl Substitutable<tst::TST> for tst::Typ {
    fn subst<S: Substitution<tst::TST>>(&self, ctx: &mut Ctx, by: &S) -> Self {
        Self::from(self.as_exp().subst(ctx, by))
    }
}
