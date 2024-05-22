use std::rc::Rc;

use crate::ast::Hole;
use crate::ast::TypeUniv;
use crate::ast::Variable;
use crate::common::*;
use crate::ctx::*;

use super::*;

impl Substitutable for Rc<Exp> {
    type Result = Rc<Exp>;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        match &**self {
            Exp::Variable(Variable { span, idx, name, .. }) => {
                match by.get_subst(ctx, ctx.idx_to_lvl(*idx)) {
                    Some(exp) => exp,
                    None => Rc::new(Exp::Variable(Variable {
                        span: *span,
                        idx: *idx,
                        name: name.clone(),
                        inferred_type: None,
                    })),
                }
            }
            Exp::TypCtor(e) => Rc::new(Exp::TypCtor(e.subst(ctx, by))),
            Exp::Call(Call { span, name, args, kind, .. }) => Rc::new(Exp::Call(Call {
                span: *span,
                kind: *kind,
                name: name.clone(),
                args: args.subst(ctx, by),
                inferred_type: None,
            })),
            Exp::DotCall(DotCall { span, kind, exp, name, args, .. }) => {
                Rc::new(Exp::DotCall(DotCall {
                    span: *span,
                    kind: *kind,
                    exp: exp.subst(ctx, by),
                    name: name.clone(),
                    args: args.subst(ctx, by),
                    inferred_type: None,
                }))
            }
            Exp::Anno(Anno { span, exp, typ, .. }) => Rc::new(Exp::Anno(Anno {
                span: *span,
                exp: exp.subst(ctx, by),
                typ: typ.subst(ctx, by),
                normalized_type: None,
            })),
            Exp::TypeUniv(TypeUniv { span }) => Rc::new(Exp::TypeUniv(TypeUniv { span: *span })),
            Exp::LocalMatch(LocalMatch { span, name, on_exp, motive, ret_typ, body, .. }) => {
                Rc::new(Exp::LocalMatch(LocalMatch {
                    span: *span,
                    ctx: None,
                    name: name.clone(),
                    on_exp: on_exp.subst(ctx, by),
                    motive: motive.subst(ctx, by),
                    ret_typ: ret_typ.subst(ctx, by),
                    body: body.subst(ctx, by),
                    inferred_type: None,
                }))
            }
            Exp::LocalComatch(LocalComatch { span, name, is_lambda_sugar, body, .. }) => {
                Rc::new(Exp::LocalComatch(LocalComatch {
                    span: *span,
                    ctx: None,
                    name: name.clone(),
                    is_lambda_sugar: *is_lambda_sugar,
                    body: body.subst(ctx, by),
                    inferred_type: None,
                }))
            }
            Exp::Hole(Hole { span, metavar, args, .. }) => Rc::new(Exp::Hole(Hole {
                span: *span,
                metavar: *metavar,
                inferred_type: None,
                inferred_ctx: None,
                args: args.subst(ctx, by),
            })),
        }
    }
}

impl Substitutable for TypCtor {
    type Result = TypCtor;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        let TypCtor { span, name, args } = self;
        TypCtor { span: *span, name: name.clone(), args: args.subst(ctx, by) }
    }
}

impl Substitutable for Motive {
    type Result = Motive;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        let Motive { span, param, ret_typ } = self;

        Motive {
            span: *span,
            param: param.clone(),
            ret_typ: ctx.bind_single((), |ctx| ret_typ.subst(ctx, &by.shift((1, 0)))),
        }
    }
}

impl Substitutable for Match {
    type Result = Match;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        let Match { span, cases, omit_absurd } = self;
        Match {
            span: *span,
            cases: cases.iter().map(|case| case.subst(ctx, by)).collect(),
            omit_absurd: *omit_absurd,
        }
    }
}

impl Substitutable for Case {
    type Result = Case;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        let Case { span, name, params, body } = self;
        ctx.bind_iter(params.params.iter(), |ctx| Case {
            span: *span,
            name: name.clone(),
            params: params.clone(),
            body: body.as_ref().map(|body| body.subst(ctx, &by.shift((1, 0)))),
        })
    }
}

impl Substitutable for Param {
    type Result = Param;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        let Param { name, typ } = self;
        Param { name: name.clone(), typ: typ.subst(ctx, by) }
    }
}

impl Substitutable for Args {
    type Result = Args;
    fn subst<S: Substitution>(&self, ctx: &mut LevelCtx, by: &S) -> Self {
        Args { args: self.args.subst(ctx, by) }
    }
}

impl<T: Substitutable> SwapWithCtx for T {
    fn swap_with_ctx(
        &self,
        ctx: &mut LevelCtx,
        fst1: usize,
        fst2: usize,
    ) -> <T as Substitutable>::Result {
        self.subst(ctx, &SwapSubst { fst1, fst2 })
    }
}

#[derive(Clone)]
struct SwapSubst {
    fst1: usize,
    fst2: usize,
}

impl Shift for SwapSubst {
    fn shift_in_range<R: ShiftRange>(&self, _range: R, _by: (isize, isize)) -> Self {
        // Since SwapSubst works with levels, it is shift-invariant
        self.clone()
    }
}

impl Substitution for SwapSubst {
    fn get_subst(&self, ctx: &LevelCtx, lvl: Lvl) -> Option<Rc<Exp>> {
        let new_lvl = if lvl.fst == self.fst1 {
            Some(Lvl { fst: self.fst2, snd: lvl.snd })
        } else if lvl.fst == self.fst2 {
            Some(Lvl { fst: self.fst1, snd: lvl.snd })
        } else {
            None
        };

        let new_ctx = ctx.swap(self.fst1, self.fst2);

        new_lvl.map(|new_lvl| {
            Rc::new(Exp::Variable(Variable {
                span: None,
                idx: new_ctx.lvl_to_idx(new_lvl),
                name: "".to_owned(),
                inferred_type: None,
            }))
        })
    }
}
