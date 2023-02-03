use std::rc::Rc;

use crate::common::*;
use crate::ctx::*;
use crate::tst;
use crate::wst;

use super::generic::*;

impl<P: Phase> Substitutable<Rc<Exp<P>>> for Rc<Exp<P>>
where
    P::InfTyp: Substitutable<Rc<Exp<P>>>,
{
    fn subst<S: Substitution<Rc<Exp<P>>>>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
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
            Exp::Match { info, name, on_exp, motive, ret_typ, body } => Rc::new(Exp::Match {
                info: info.clone(),
                name: name.clone(),
                on_exp: on_exp.subst(ctx, by),
                motive: motive.subst(ctx, by),
                ret_typ: ret_typ.subst(ctx, by),
                body: body.subst(ctx, by),
            }),
            Exp::Comatch { info, name, body } => Rc::new(Exp::Comatch {
                info: info.clone(),
                name: name.clone(),
                body: body.subst(ctx, by),
            }),
            Exp::Hole { info } => Rc::new(Exp::Hole { info: info.clone() }),
        }
    }
}

impl<P: Phase> Substitutable<Rc<Exp<P>>> for Motive<P>
where
    P::InfTyp: Substitutable<Rc<Exp<P>>>,
{
    fn subst<S: Substitution<Rc<Exp<P>>>>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        let Motive { info, param, ret_typ } = self;

        Motive {
            info: info.clone(),
            param: param.clone(),
            ret_typ: ctx.bind_single((), |ctx| ret_typ.subst(ctx, &by.shift((1, 0)))),
        }
    }
}

impl<P: Phase> Substitutable<Rc<Exp<P>>> for Match<P>
where
    P::InfTyp: Substitutable<Rc<Exp<P>>>,
{
    fn subst<S: Substitution<Rc<Exp<P>>>>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        let Match { info, cases } = self;
        Match { info: info.clone(), cases: cases.iter().map(|case| case.subst(ctx, by)).collect() }
    }
}

impl<P: Phase> Substitutable<Rc<Exp<P>>> for Comatch<P>
where
    P::InfTyp: Substitutable<Rc<Exp<P>>>,
{
    fn subst<S: Substitution<Rc<Exp<P>>>>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        let Comatch { info, cases } = self;
        Comatch {
            info: info.clone(),
            cases: cases.iter().map(|cocase| cocase.subst(ctx, by)).collect(),
        }
    }
}

impl<P: Phase> Substitutable<Rc<Exp<P>>> for Case<P>
where
    P::InfTyp: Substitutable<Rc<Exp<P>>>,
{
    fn subst<S: Substitution<Rc<Exp<P>>>>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        let Case { info, name, args, body } = self;
        ctx.bind_iter(args.params.iter(), |ctx| Case {
            info: info.clone(),
            name: name.clone(),
            args: args.clone(),
            body: body.as_ref().map(|body| body.subst(ctx, &by.shift((1, 0)))),
        })
    }
}

impl<P: Phase> Substitutable<Rc<Exp<P>>> for Cocase<P>
where
    P::InfTyp: Substitutable<Rc<Exp<P>>>,
{
    fn subst<S: Substitution<Rc<Exp<P>>>>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        let Cocase { info, name, params: args, body } = self;
        ctx.bind_iter(args.params.iter(), |ctx| Cocase {
            info: info.clone(),
            name: name.clone(),
            params: args.clone(),
            body: body.as_ref().map(|body| body.subst(ctx, &by.shift((1, 0)))),
        })
    }
}

impl<P: Phase> Substitutable<Rc<Exp<P>>> for TypApp<P>
where
    P::InfTyp: Substitutable<Rc<Exp<P>>>,
{
    fn subst<S: Substitution<Rc<Exp<P>>>>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        let TypApp { info, name, args: subst } = self;
        TypApp { info: info.clone(), name: name.clone(), args: subst.subst(ctx, by) }
    }
}

impl<P: Phase> Substitutable<Rc<Exp<P>>> for Param<P>
where
    P::InfTyp: Substitutable<Rc<Exp<P>>>,
{
    fn subst<S: Substitution<Rc<Exp<P>>>>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        let Param { name, typ } = self;
        Param { name: name.clone(), typ: typ.subst(ctx, by) }
    }
}

impl<P: Phase<VarName = Ident>, T: Substitutable<Rc<Exp<P>>>> SwapWithCtx<P> for T
where
    P::TypeInfo: Default,
{
    fn swap_with_ctx(&self, ctx: &mut LevelCtx, fst1: usize, fst2: usize) -> Self {
        self.subst(ctx, &SwapSubst { fst1, fst2 })
    }
}

#[derive(Clone)]
struct SwapSubst {
    fst1: usize,
    fst2: usize,
}

impl ShiftInRange for SwapSubst {
    fn shift_in_range<R: ShiftRange>(&self, _range: R, _by: (isize, isize)) -> Self {
        // Since SwapSubst works with levels, it is shift-invariant
        self.clone()
    }
}

impl<P: Phase<VarName = Ident>> Substitution<Rc<Exp<P>>> for SwapSubst
where
    P::TypeInfo: Default,
{
    fn get_subst(&self, ctx: &LevelCtx, lvl: Lvl) -> Option<Rc<Exp<P>>> {
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

impl Substitutable<Rc<tst::Exp>> for tst::Typ {
    fn subst<S: Substitution<Rc<tst::Exp>>>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        Self::from(self.as_exp().subst(ctx, by))
    }
}

impl Substitutable<Rc<wst::Exp>> for wst::Typ {
    fn subst<S: Substitution<Rc<wst::Exp>>>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        Self::from(self.as_exp().subst(ctx, by))
    }
}
