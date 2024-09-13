use std::rc::Rc;

use codespan::Span;
use num_bigint::BigUint;

use miette_util::ToMiette;
use parser::cst;
use parser::cst::exp::BindingSite;
use parser::cst::ident::Ident;
use syntax::ast;
use syntax::ast::lookup_table::DeclKind;
use syntax::ast::Hole;
use syntax::ast::TypeUniv;
use syntax::ast::Variable;

use super::Lower;
use crate::ctx::*;
use crate::result::*;

impl Lower for cst::exp::Exp {
    type Target = ast::Exp;

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

fn lower_telescope_inst<T, F: FnOnce(&mut Ctx, ast::TelescopeInst) -> Result<T, LoweringError>>(
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
            let param_out =
                ast::ParamInst { span: Some(span), info: None, name: name.id, typ: None };
            params_out.push(param_out);
            Ok(params_out)
        },
        |ctx, params| f(ctx, params.map(|params| ast::TelescopeInst { params })?),
    )
}

impl Lower for cst::exp::Arg {
    type Target = ast::Arg;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        match self {
            cst::exp::Arg::UnnamedArg(e) => Ok(ast::Arg::UnnamedArg(e.lower(ctx)?)),
            cst::exp::Arg::NamedArg(_, _) => Err(LoweringError::Impossible {
                message: "Named arguments not yet implemented".to_owned(),
                span: None,
            }),
        }
    }
}

impl Lower for cst::exp::Case<cst::exp::Pattern> {
    type Target = ast::Case;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Case { span, pattern, body } = self;

        lower_telescope_inst(&pattern.params, ctx, |ctx, params| {
            Ok(ast::Case {
                span: Some(*span),
                pattern: ast::Pattern {
                    is_copattern: false,
                    name: pattern.name.id.clone(),
                    params,
                },
                body: body.lower(ctx)?,
            })
        })
    }
}

impl Lower for cst::exp::Case<cst::exp::Copattern> {
    type Target = ast::Case;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Case { span, pattern, body } = self;

        lower_telescope_inst(&pattern.params, ctx, |ctx, params| {
            Ok(ast::Case {
                span: Some(*span),
                pattern: ast::Pattern { is_copattern: true, name: pattern.name.id.clone(), params },
                body: body.lower(ctx)?,
            })
        })
    }
}

impl Lower for cst::exp::Call {
    type Target = ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Call { span, name, args } = self;

        // If we find the identifier in the local context then we have to lower
        // it to a variable.
        if let Some(lvl) = ctx.lookup_local(name) {
            return Ok(ast::Exp::Variable(Variable {
                span: Some(*span),
                idx: ctx.level_to_index(lvl),
                name: name.id.clone(),
                inferred_type: None,
            }));
        }

        // If we find the identifier in the global context then we have to lower
        // it to a call or a type constructor.
        if let Some(meta) = ctx.lookup_global(name) {
            match meta.kind() {
                DeclKind::Data | DeclKind::Codata => {
                    return Ok(ast::Exp::TypCtor(ast::TypCtor {
                        span: Some(*span),
                        name: name.id.to_owned(),
                        args: ast::Args { args: args.lower(ctx)? },
                    }))
                }
                DeclKind::Def | DeclKind::Dtor => {
                    return Err(LoweringError::MustUseAsDtor {
                        name: name.to_owned(),
                        span: span.to_miette(),
                    })
                }
                DeclKind::Ctor => {
                    return Ok(ast::Exp::Call(ast::Call {
                        span: Some(*span),
                        kind: ast::CallKind::Constructor,
                        name: name.id.to_owned(),
                        args: ast::Args { args: args.lower(ctx)? },
                        inferred_type: None,
                    }))
                }
                DeclKind::Codef => {
                    return Ok(ast::Exp::Call(ast::Call {
                        span: Some(*span),
                        kind: ast::CallKind::Codefinition,
                        name: name.id.to_owned(),
                        args: ast::Args { args: args.lower(ctx)? },
                        inferred_type: None,
                    }))
                }
                DeclKind::Let => {
                    return Ok(ast::Exp::Call(ast::Call {
                        span: Some(*span),
                        kind: ast::CallKind::LetBound,
                        name: name.id.to_owned(),
                        args: ast::Args { args: args.lower(ctx)? },
                        inferred_type: None,
                    }))
                }
            }
        };

        // If we find the identifier neither in the local nor the global context then we have
        // to throw an error.
        Err(LoweringError::UndefinedIdent { name: name.to_owned(), span: span.to_miette() })
    }
}

impl Lower for cst::exp::DotCall {
    type Target = ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::DotCall { span, exp, name, args } = self;

        match ctx.lookup_global(name) {
            Some(meta) => match meta.kind() {
                DeclKind::Dtor => Ok(ast::Exp::DotCall(ast::DotCall {
                    span: Some(*span),
                    kind: ast::DotCallKind::Destructor,
                    exp: exp.lower(ctx)?,
                    name: name.id.clone(),
                    args: ast::Args { args: args.lower(ctx)? },
                    inferred_type: None,
                })),
                DeclKind::Def => Ok(ast::Exp::DotCall(ast::DotCall {
                    span: Some(*span),
                    kind: ast::DotCallKind::Definition,
                    exp: exp.lower(ctx)?,
                    name: name.id.clone(),
                    args: ast::Args { args: args.lower(ctx)? },
                    inferred_type: None,
                })),
                _ => Err(LoweringError::CannotUseAsDtor {
                    name: name.clone(),
                    span: span.to_miette(),
                }),
            },
            None => {
                Err(LoweringError::UndefinedIdent { name: name.to_owned(), span: span.to_miette() })
            }
        }
    }
}

impl Lower for cst::exp::Anno {
    type Target = ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Anno { span, exp, typ } = self;
        Ok(ast::Anno {
            span: Some(*span),
            exp: exp.lower(ctx)?,
            typ: typ.lower(ctx)?,
            normalized_type: None,
        }
        .into())
    }
}

impl Lower for cst::exp::TypeUniv {
    type Target = ast::Exp;

    fn lower(&self, _ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::TypeUniv { span } = self;
        Ok(TypeUniv { span: Some(*span) }.into())
    }
}

impl Lower for cst::exp::LocalMatch {
    type Target = ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::LocalMatch { span, name, on_exp, motive, cases } = self;
        Ok(ast::LocalMatch {
            span: Some(*span),
            ctx: None,
            name: ctx.unique_label(name.to_owned(), span)?,
            on_exp: on_exp.lower(ctx)?,
            motive: motive.lower(ctx)?,
            ret_typ: None,
            cases: cases.lower(ctx)?,
            inferred_type: None,
        }
        .into())
    }
}

impl Lower for cst::exp::LocalComatch {
    type Target = ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::LocalComatch { span, name, is_lambda_sugar, cases } = self;
        Ok(ast::LocalComatch {
            span: Some(*span),
            ctx: None,
            name: ctx.unique_label(name.to_owned(), span)?,
            is_lambda_sugar: *is_lambda_sugar,
            cases: cases.lower(ctx)?,
            inferred_type: None,
        }
        .into())
    }
}

impl Lower for cst::exp::Hole {
    type Target = ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Hole { span } = self;
        let mv = ctx.fresh_metavar();
        let args = ctx.subst_from_ctx();
        Ok(Hole { span: Some(*span), metavar: mv, inferred_type: None, inferred_ctx: None, args }
            .into())
    }
}

impl Lower for cst::exp::NatLit {
    type Target = ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::NatLit { span, val } = self;

        // We have to check whether "Z" is declared as a constructor or codefinition.
        // We assume that if Z exists, then S exists as well and is of the same kind.
        let z_kind = ctx
            .lookup_top_level_decl(&Ident { id: "Z".to_string() }, span)
            .map_err(|_| LoweringError::NatLiteralCannotBeDesugared { span: span.to_miette() })?;
        let call_kind = match z_kind {
            ast::lookup_table::DeclMeta::Codef => ast::CallKind::Codefinition,
            ast::lookup_table::DeclMeta::Ctor { .. } => ast::CallKind::Constructor,
            _ => return Err(LoweringError::NatLiteralCannotBeDesugared { span: span.to_miette() }),
        };

        let mut out = ast::Exp::Call(ast::Call {
            span: Some(*span),
            kind: call_kind,
            name: "Z".to_owned(),
            args: ast::Args { args: vec![] },
            inferred_type: None,
        });

        let mut i = BigUint::from(0usize);

        while &i != val {
            i += 1usize;
            out = ast::Exp::Call(ast::Call {
                span: Some(*span),
                kind: call_kind,
                name: "S".to_owned(),
                args: ast::Args { args: vec![ast::Arg::UnnamedArg(Rc::new(out))] },
                inferred_type: None,
            });
        }

        Ok(out)
    }
}

impl Lower for cst::exp::Fun {
    type Target = ast::Exp;
    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Fun { span, from, to } = self;
        Ok(ast::TypCtor {
            span: Some(*span),
            name: "Fun".to_owned(),
            args: ast::Args {
                args: vec![
                    ast::Arg::UnnamedArg(from.lower(ctx)?),
                    ast::Arg::UnnamedArg(to.lower(ctx)?),
                ],
            },
        }
        .into())
    }
}

impl Lower for cst::exp::Lam {
    type Target = ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Lam { span, var, body } = self;

        let case = cst::exp::Case {
            span: *span,
            pattern: cst::exp::Copattern {
                name: Ident { id: "ap".to_owned() },
                params: vec![
                    cst::exp::BindingSite::Wildcard { span: Default::default() },
                    cst::exp::BindingSite::Wildcard { span: Default::default() },
                    var.clone(),
                ],
            },
            body: Some(body.clone()),
        };
        let comatch = cst::exp::Exp::LocalComatch(cst::exp::LocalComatch {
            span: *span,
            name: None,
            is_lambda_sugar: true,
            cases: vec![case],
        });
        comatch.lower(ctx)
    }
}

fn bs_to_name(bs: &cst::exp::BindingSite) -> Ident {
    match bs {
        BindingSite::Var { name, .. } => name.clone(),
        BindingSite::Wildcard { .. } => Ident { id: "_".to_owned() },
    }
}

fn bs_to_span(bs: &cst::exp::BindingSite) -> Span {
    match bs {
        BindingSite::Var { span, .. } => *span,
        BindingSite::Wildcard { span } => *span,
    }
}

impl Lower for cst::exp::Motive {
    type Target = ast::Motive;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Motive { span, param, ret_typ } = self;

        Ok(ast::Motive {
            span: Some(*span),
            param: ast::ParamInst {
                span: Some(bs_to_span(param)),
                info: None,
                name: bs_to_name(param).id,
                typ: None,
            },
            ret_typ: ctx.bind_single(param, |ctx| ret_typ.lower(ctx))?,
        })
    }
}
