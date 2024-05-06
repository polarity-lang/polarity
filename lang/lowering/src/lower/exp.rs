use std::rc::Rc;

use codespan::Span;
use num_bigint::BigUint;

use miette_util::ToMiette;
use parser::cst;
use parser::cst::exp::BindingSite;
use parser::cst::exp::Ident;
use syntax::ctx::BindContext;
use syntax::generic;
use syntax::generic::lookup_table::DeclKind;
use syntax::generic::Hole;
use syntax::generic::TypeUniv;
use syntax::generic::Variable;

use super::Lower;
use crate::ctx::*;
use crate::result::*;

impl Lower for cst::exp::Exp {
    type Target = generic::Exp;

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
    type Target = generic::Match;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Match { span, cases, omit_absurd } = self;

        Ok(generic::Match {
            span: Some(*span),
            cases: cases.lower(ctx)?,
            omit_absurd: *omit_absurd,
        })
    }
}

fn lower_telescope_inst<
    T,
    F: FnOnce(&mut Ctx, generic::TelescopeInst) -> Result<T, LoweringError>,
>(
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
            let param_out = generic::ParamInst { span: Some(span), info: None, name, typ: None };
            params_out.push(param_out);
            Ok(params_out)
        },
        |ctx, params| f(ctx, params.map(|params| generic::TelescopeInst { params })?),
    )
}

impl Lower for cst::exp::Case {
    type Target = generic::Case;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Case { span, name, params, body } = self;

        lower_telescope_inst(params, ctx, |ctx, params| {
            Ok(generic::Case {
                span: Some(*span),
                name: name.clone(),
                params,
                body: body.lower(ctx)?,
            })
        })
    }
}

impl Lower for cst::exp::Call {
    type Target = generic::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Call { span, name, args } = self;
        match ctx.lookup(name, span)? {
            Elem::Bound(lvl) => Ok(generic::Exp::Variable(Variable {
                span: Some(*span),
                idx: ctx.level_to_index(lvl),
                name: name.clone(),
                inferred_type: None,
            })),
            Elem::Decl(meta) => match meta.kind() {
                DeclKind::Data | DeclKind::Codata => Ok(generic::Exp::TypCtor(generic::TypCtor {
                    span: Some(*span),
                    name: name.to_owned(),
                    args: generic::Args { args: args.lower(ctx)? },
                })),
                DeclKind::Def | DeclKind::Dtor => Err(LoweringError::MustUseAsDtor {
                    name: name.to_owned(),
                    span: span.to_miette(),
                }),
                DeclKind::Ctor => Ok(generic::Exp::Call(generic::Call {
                    span: Some(*span),
                    kind: generic::CallKind::Constructor,
                    name: name.to_owned(),
                    args: generic::Args { args: args.lower(ctx)? },
                    inferred_type: None,
                })),
                DeclKind::Codef => Ok(generic::Exp::Call(generic::Call {
                    span: Some(*span),
                    kind: generic::CallKind::Codefinition,
                    name: name.to_owned(),
                    args: generic::Args { args: args.lower(ctx)? },
                    inferred_type: None,
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
    type Target = generic::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::DotCall { span, exp, name, args } = self;

        match ctx.lookup(name, span)? {
            Elem::Bound(_) => {
                Err(LoweringError::CannotUseAsDtor { name: name.clone(), span: span.to_miette() })
            }
            Elem::Decl(meta) => match meta.kind() {
                DeclKind::Dtor => Ok(generic::Exp::DotCall(generic::DotCall {
                    span: Some(*span),
                    kind: generic::DotCallKind::Destructor,
                    exp: exp.lower(ctx)?,
                    name: name.clone(),
                    args: generic::Args { args: args.lower(ctx)? },
                    inferred_type: None,
                })),
                DeclKind::Def => Ok(generic::Exp::DotCall(generic::DotCall {
                    span: Some(*span),
                    kind: generic::DotCallKind::Definition,
                    exp: exp.lower(ctx)?,
                    name: name.clone(),
                    args: generic::Args { args: args.lower(ctx)? },
                    inferred_type: None,
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
    type Target = generic::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Anno { span, exp, typ } = self;
        Ok(generic::Exp::Anno(generic::Anno {
            span: Some(*span),
            exp: exp.lower(ctx)?,
            typ: typ.lower(ctx)?,
            normalized_type: None,
        }))
    }
}

impl Lower for cst::exp::TypeUniv {
    type Target = generic::Exp;

    fn lower(&self, _ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::TypeUniv { span } = self;
        Ok(generic::Exp::TypeUniv(TypeUniv { span: Some(*span) }))
    }
}

impl Lower for cst::exp::LocalMatch {
    type Target = generic::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::LocalMatch { span, name, on_exp, motive, body } = self;
        Ok(generic::Exp::LocalMatch(generic::LocalMatch {
            span: Some(*span),
            ctx: None,
            name: ctx.unique_label(name.to_owned(), span)?,
            on_exp: on_exp.lower(ctx)?,
            motive: motive.lower(ctx)?,
            ret_typ: None,
            body: body.lower(ctx)?,
            inferred_type: None,
        }))
    }
}

impl Lower for cst::exp::LocalComatch {
    type Target = generic::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::LocalComatch { span, name, is_lambda_sugar, body } = self;
        Ok(generic::Exp::LocalComatch(generic::LocalComatch {
            span: Some(*span),
            ctx: None,
            name: ctx.unique_label(name.to_owned(), span)?,
            is_lambda_sugar: *is_lambda_sugar,
            body: body.lower(ctx)?,
            inferred_type: None,
        }))
    }
}

impl Lower for cst::exp::Hole {
    type Target = generic::Exp;

    fn lower(&self, _ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Hole { span } = self;
        Ok(generic::Exp::Hole(Hole { span: Some(*span), inferred_type: None, inferred_ctx: None }))
    }
}

impl Lower for cst::exp::NatLit {
    type Target = generic::Exp;

    fn lower(&self, _ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::NatLit { span, val } = self;
        let mut out = generic::Exp::Call(generic::Call {
            span: Some(*span),
            kind: generic::CallKind::Constructor,
            name: "Z".to_owned(),
            args: generic::Args { args: vec![] },
            inferred_type: None,
        });

        let mut i = BigUint::from(0usize);

        while &i != val {
            i += 1usize;
            out = generic::Exp::Call(generic::Call {
                span: Some(*span),
                kind: generic::CallKind::Constructor,
                name: "S".to_owned(),
                args: generic::Args { args: vec![Rc::new(out)] },
                inferred_type: None,
            });
        }

        Ok(out)
    }
}

impl Lower for cst::exp::Fun {
    type Target = generic::Exp;
    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Fun { span, from, to } = self;
        Ok(generic::Exp::TypCtor(generic::TypCtor {
            span: Some(*span),
            name: "Fun".to_owned(),
            args: generic::Args { args: vec![from.lower(ctx)?, to.lower(ctx)?] },
        }))
    }
}

impl Lower for cst::exp::Lam {
    type Target = generic::Exp;

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
    type Target = generic::Motive;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Motive { span, param, ret_typ } = self;

        Ok(generic::Motive {
            span: Some(*span),
            param: generic::ParamInst {
                span: Some(bs_to_span(param)),
                info: None,
                name: bs_to_name(param),
                typ: None,
            },
            ret_typ: ctx.bind_single(param, |ctx| ret_typ.lower(ctx))?,
        })
    }
}
