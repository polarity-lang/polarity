use std::rc::Rc;

use codespan::Span;
use num_bigint::BigUint;

use miette_util::ToMiette;
use parser::cst;
use parser::cst::exp::BindingSite;
use parser::cst::exp::Ident;
use syntax::ctx::BindContext;
use syntax::generic::lookup_table::DeclKind;
use syntax::generic::Hole;
use syntax::generic::TypeUniv;
use syntax::generic::Variable;
use syntax::ust;

use super::Lower;
use crate::ctx::*;
use crate::result::*;

impl Lower for cst::exp::Exp {
    type Target = ust::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        match self {
            cst::exp::Exp::Call(e) => e.lower(ctx),
            cst::exp::Exp::DotCall(e) => e.lower(ctx),
            cst::exp::Exp::Anno(e) => e.lower(ctx),
            cst::exp::Exp::TypeUniv(e) => e.lower(ctx),
            cst::exp::Exp::LocalMatch(e) => e.lower(ctx),
            cst::exp::Exp::LocalComatch(e) => e.lower(ctx),
            cst::exp::Exp::Hole(e) => e.lower(ctx),
            cst::exp::Exp::NatLit(e) => e.lower(ctx),
            cst::exp::Exp::Fun(e) => e.lower(ctx),
            cst::exp::Exp::Lam(e) => e.lower(ctx),
        }
    }
}

impl Lower for cst::exp::Match {
    type Target = ust::Match;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Match { span, cases, omit_absurd } = self;

        Ok(ust::Match { span: Some(*span), cases: cases.lower(ctx)?, omit_absurd: *omit_absurd })
    }
}

fn lower_telescope_inst<T, F: FnOnce(&mut Ctx, ust::TelescopeInst) -> Result<T, LoweringError>>(
    tel_inst: &[cst::exp::BindingSite],
    ctx: &mut Ctx,
    f: F,
) -> Result<T, LoweringError> {
    ctx.bind_fold(
        tel_inst.iter(),
        Ok(vec![]),
        |_ctx, params_out, param| {
            let mut params_out = params_out?;
            let span = bs_to_span(param);
            let name = bs_to_name(param);
            let param_out = ust::ParamInst { span: Some(span), info: (), name, typ: None };
            params_out.push(param_out);
            Ok(params_out)
        },
        |ctx, params| f(ctx, params.map(|params| ust::TelescopeInst { params })?),
    )
}

impl Lower for cst::exp::Case {
    type Target = ust::Case;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Case { span, name, params, body } = self;

        lower_telescope_inst(params, ctx, |ctx, params| {
            Ok(ust::Case { span: Some(*span), name: name.clone(), params, body: body.lower(ctx)? })
        })
    }
}

impl Lower for cst::exp::Call {
    type Target = ust::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Call { span, name, args } = self;
        match ctx.lookup(name, span)? {
            Elem::Bound(lvl) => Ok(ust::Exp::Variable(Variable {
                span: Some(*span),
                idx: ctx.level_to_index(lvl),
                name: name.clone(),
                inferred_type: None,
            })),
            Elem::Decl(meta) => match meta.kind() {
                DeclKind::Data | DeclKind::Codata => Ok(ust::Exp::TypCtor(ust::TypCtor {
                    span: Some(*span),
                    info: (),
                    name: name.to_owned(),
                    args: ust::Args { args: args.lower(ctx)? },
                })),
                DeclKind::Def | DeclKind::Dtor => Err(LoweringError::MustUseAsDtor {
                    name: name.to_owned(),
                    span: span.to_miette(),
                }),
                DeclKind::Codef | DeclKind::Ctor => Ok(ust::Exp::Call(ust::Call {
                    span: Some(*span),
                    info: (),
                    name: name.to_owned(),
                    args: ust::Args { args: args.lower(ctx)? },
                })),
                DeclKind::Let => Err(LoweringError::Impossible {
                    message: "Referencing top-level let definitions is not implemented, yet"
                        .to_owned(),
                    span: Some(span.to_miette()),
                }),
            },
        }
    }
}

impl Lower for cst::exp::DotCall {
    type Target = ust::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::DotCall { span, exp, name, args } = self;

        match ctx.lookup(name, span)? {
            Elem::Bound(_) => {
                Err(LoweringError::CannotUseAsDtor { name: name.clone(), span: span.to_miette() })
            }
            Elem::Decl(meta) => match meta.kind() {
                DeclKind::Def | DeclKind::Dtor => Ok(ust::Exp::DotCall(ust::DotCall {
                    span: Some(*span),
                    info: (),
                    exp: exp.lower(ctx)?,
                    name: name.clone(),
                    args: ust::Args { args: args.lower(ctx)? },
                })),
                _ => Err(LoweringError::CannotUseAsDtor {
                    name: name.clone(),
                    span: span.to_miette(),
                }),
            },
        }
    }
}

impl Lower for cst::exp::Anno {
    type Target = ust::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Anno { span, exp, typ } = self;
        Ok(ust::Exp::Anno(ust::Anno {
            span: Some(*span),
            info: (),
            exp: exp.lower(ctx)?,
            typ: typ.lower(ctx)?,
        }))
    }
}

impl Lower for cst::exp::TypeUniv {
    type Target = ust::Exp;

    fn lower(&self, _ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::TypeUniv { span } = self;
        Ok(ust::Exp::TypeUniv(TypeUniv { span: Some(*span) }))
    }
}

impl Lower for cst::exp::LocalMatch {
    type Target = ust::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::LocalMatch { span, name, on_exp, motive, body } = self;
        Ok(ust::Exp::LocalMatch(ust::LocalMatch {
            span: Some(*span),
            info: (),
            ctx: None,
            name: ctx.unique_label(name.to_owned(), span)?,
            on_exp: on_exp.lower(ctx)?,
            motive: motive.lower(ctx)?,
            ret_typ: None,
            body: body.lower(ctx)?,
        }))
    }
}

impl Lower for cst::exp::LocalComatch {
    type Target = ust::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::LocalComatch { span, name, is_lambda_sugar, body } = self;
        Ok(ust::Exp::LocalComatch(ust::LocalComatch {
            span: Some(*span),
            info: (),
            ctx: None,
            name: ctx.unique_label(name.to_owned(), span)?,
            is_lambda_sugar: *is_lambda_sugar,
            body: body.lower(ctx)?,
        }))
    }
}

impl Lower for cst::exp::Hole {
    type Target = ust::Exp;

    fn lower(&self, _ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Hole { span } = self;
        Ok(ust::Exp::Hole(Hole { span: Some(*span), inferred_type: None, inferred_ctx: None }))
    }
}

impl Lower for cst::exp::NatLit {
    type Target = ust::Exp;

    fn lower(&self, _ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::NatLit { span, val } = self;
        let mut out = ust::Exp::Call(ust::Call {
            span: Some(*span),
            info: (),
            name: "Z".to_owned(),
            args: ust::Args { args: vec![] },
        });

        let mut i = BigUint::from(0usize);

        while &i != val {
            i += 1usize;
            out = ust::Exp::Call(ust::Call {
                span: Some(*span),
                info: (),
                name: "S".to_owned(),
                args: ust::Args { args: vec![Rc::new(out)] },
            });
        }

        Ok(out)
    }
}

impl Lower for cst::exp::Fun {
    type Target = ust::Exp;
    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Fun { span, from, to } = self;
        Ok(ust::Exp::TypCtor(ust::TypCtor {
            span: Some(*span),
            info: (),
            name: "Fun".to_owned(),
            args: ust::Args { args: vec![from.lower(ctx)?, to.lower(ctx)?] },
        }))
    }
}

impl Lower for cst::exp::Lam {
    type Target = ust::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Lam { span, var, body } = self;
        let comatch = cst::exp::Exp::LocalComatch(cst::exp::LocalComatch {
            span: *span,
            name: None,
            is_lambda_sugar: true,
            body: cst::exp::Match {
                span: *span,
                cases: vec![cst::exp::Case {
                    span: *span,
                    name: "ap".to_owned(),
                    params: vec![
                        cst::exp::BindingSite::Wildcard { span: Default::default() },
                        cst::exp::BindingSite::Wildcard { span: Default::default() },
                        var.clone(),
                    ],
                    body: Some(body.clone()),
                }],
                omit_absurd: false,
            },
        });
        comatch.lower(ctx)
    }
}

fn bs_to_name(bs: &cst::exp::BindingSite) -> Ident {
    match bs {
        BindingSite::Var { name, .. } => name.clone(),
        BindingSite::Wildcard { .. } => "_".to_owned(),
    }
}

fn bs_to_span(bs: &cst::exp::BindingSite) -> Span {
    match bs {
        BindingSite::Var { span, .. } => *span,
        BindingSite::Wildcard { span } => *span,
    }
}

impl Lower for cst::exp::Motive {
    type Target = ust::Motive;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Motive { span, param, ret_typ } = self;

        Ok(ust::Motive {
            span: Some(*span),
            param: ust::ParamInst {
                span: Some(bs_to_span(param)),
                info: (),
                name: bs_to_name(param),
                typ: None,
            },
            ret_typ: ctx.bind_single(param, |ctx| ret_typ.lower(ctx))?,
        })
    }
}
