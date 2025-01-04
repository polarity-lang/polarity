use ast::IdBound;
use ast::MetaVarKind;
use ast::VarBind;
use ast::VarBound;
use codespan::Span;
use num_bigint::BigUint;

use ast::Hole;
use ast::TypeUniv;
use ast::Variable;
use miette_util::ToMiette;
use parser::cst;
use parser::cst::decls::Telescope;
use parser::cst::exp::BindingSite;
use parser::cst::ident::Ident;

use crate::ctx::*;
use crate::result::*;
use crate::symbol_table::DeclMeta;

use super::Lower;

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
            let param_out = ast::ParamInst {
                span: Some(span),
                info: None,
                name: VarBind { span: Some(span), id: name.id.clone() },
                typ: None,
                erased: false,
            };
            params_out.push(param_out);
            Ok(params_out)
        },
        |ctx, params| f(ctx, params.map(|params| ast::TelescopeInst { params })?),
    )
}
/// Lowers a list of arguments, ensuring that named arguments match the expected parameter names.
///
/// This function processes a mix of named and unnamed arguments provided by the user (`given`)
/// and matches them against the expected parameters (`expected`). It handles implicit parameters
/// by inserting fresh metavariables when necessary and ensures that named arguments correspond to the correct
/// parameters as declared.
///
/// # Parameters
///
/// - `given`: A slice of `cst::exp::Arg` representing the arguments provided by the user.
/// - `expected`: A `Telescope` containing the expected parameters.
/// - `ctx`: A mutable reference to the current context (`Ctx`), used for tracking variables and generating fresh metavariables.
///
/// # Returns
///
/// - `Ok(ast::Args)`: The successfully lowered arguments.
/// - `Err(LoweringError)`: An error indicating issues such as missing arguments, too many arguments,
///   mismatched named arguments, or improper use of wildcards.
///
/// # Errors
///
/// This function may return a `LoweringError` in the following cases:
///
/// - **MissingArgForParam**: A required argument is missing for a parameter.
/// - **TooManyArgs**: More arguments are provided than there are expected parameters.
/// - **MismatchedNamedArgs**: A named argument does not match the expected parameter name.
/// - **NamedArgForWildcard**: A named argument is provided for a wildcard parameter, which is not allowed.
///
/// # Example
///
/// ```text
/// data Bool { True, False }
///
/// data List {
///     Nil,
///     Cons(head: Bool, tail: List)
/// }
///
/// let example1 : List {
///     Cons(x := True, xs := Nil)
/// }
/// ```
///
/// In this example, an error is thrown because the named arguments `x` and `xs` do not match
/// the expected parameter names `head` and `tail`.
fn lower_args(
    span: Span,
    given: &[cst::exp::Arg],
    expected: Telescope,
    ctx: &mut Ctx,
) -> Result<ast::Args, LoweringError> {
    let mut args_out = vec![];

    // Ensure that the number of given arguments does not exceed the number of expected parameters.
    // Some expected parameters might be implicit and not require corresponding given arguments.
    if given.len() > expected.len() {
        // The unwrap is safe because in this branch there must be at least one given.
        let err = LoweringError::TooManyArgs { span: given.first().unwrap().span().to_miette() };
        return Err(err);
    }

    // Create a peekable iterator over the given arguments to allow lookahead.
    let mut given_iter = given.iter().peekable();

    /// Processes a single argument, matching it against the expected parameter binding site.
    ///
    /// This function consumes one argument from the `given` iterator and attempts to match it with the
    /// expected binding site (`expected_bs`). It handles both named and unnamed arguments and ensures
    /// that named arguments match the expected parameter names.
    ///
    /// # Parameters
    ///
    /// - `span`: The source span of the given argument list.
    /// - `given`: A mutable iterator over the given arguments.
    /// - `expected_bs`: The binding site of the expected parameter.
    /// - `args_out`: A mutable vector to collect the lowered arguments.
    /// - `ctx`: A mutable reference to the current context.
    ///
    /// # Returns
    ///
    /// - `Ok(())`: The argument was successfully processed and added to `args_out`.
    /// - `Err(LoweringError)`: An error occurred while processing the argument.
    fn pop_arg<'a>(
        span: Span,
        given: &mut impl Iterator<Item = &'a cst::exp::Arg>,
        expected_bs: &BindingSite,
        args_out: &mut Vec<ast::Arg>,
        ctx: &mut Ctx,
    ) -> Result<(), LoweringError> {
        let Some(arg) = given.next() else {
            return Err(LoweringError::MissingArgForParam {
                expected: bs_to_name(expected_bs).to_owned(),
                span: span.to_miette(),
            });
        };
        match arg {
            cst::exp::Arg::UnnamedArg(exp) => {
                args_out.push(ast::Arg::UnnamedArg { arg: exp.lower(ctx)?, erased: false });
            }
            cst::exp::Arg::NamedArg(name, exp) => {
                let expected_name = match &expected_bs {
                    BindingSite::Var { name, .. } => name,
                    BindingSite::Wildcard { span } => {
                        return Err(LoweringError::NamedArgForWildcard {
                            given: name.clone(),
                            span: span.to_miette(),
                        });
                    }
                };
                if name.id != expected_name.id {
                    return Err(LoweringError::MismatchedNamedArgs {
                        given: name.to_owned(),
                        expected: expected_name.to_owned(),
                        span: exp.span().to_miette(),
                    });
                }
                let name = VarBound { span: Some(name.span), id: name.id.clone() };
                args_out.push(ast::Arg::NamedArg { name, arg: exp.lower(ctx)?, erased: false });
            }
        }
        Ok(())
    }

    for expected_param in expected.0.iter() {
        // Each parameter can have multiple names (e.g., aliases).
        let names_iter = std::iter::once(&expected_param.name).chain(expected_param.names.iter());
        for expected_bs in names_iter {
            if expected_param.implicit {
                if let Some(cst::exp::Arg::NamedArg(given_name, exp)) = given_iter.peek() {
                    let BindingSite::Var { name: expected_name, .. } = &expected_bs else {
                        return Err(LoweringError::NamedArgForWildcard {
                            given: given_name.clone(),
                            span: exp.span().to_miette(),
                        });
                    };
                    if expected_name == given_name {
                        pop_arg(span, &mut given_iter, expected_bs, &mut args_out, ctx)?;
                        continue;
                    }
                }

                let mv = ctx.fresh_metavar(Some(span), MetaVarKind::Inserted);
                let args = ctx.subst_from_ctx();
                let hole = Hole {
                    span: None,
                    kind: ast::MetaVarKind::Inserted,
                    metavar: mv,
                    inferred_type: None,
                    inferred_ctx: None,
                    args,
                    solution: None,
                };

                args_out.push(ast::Arg::InsertedImplicitArg { hole, erased: false });
            } else {
                pop_arg(span, &mut given_iter, expected_bs, &mut args_out, ctx)?;
            }
        }
    }

    // Check for any extra arguments that were not matched to parameters.
    if let Some(extra_arg) = given_iter.next() {
        return Err(LoweringError::TooManyArgs { span: extra_arg.span().to_miette() });
    }

    // All arguments have been successfully processed.
    Ok(ast::Args { args: args_out })
}

impl Lower for cst::exp::Case<cst::exp::Pattern> {
    type Target = ast::Case;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Case { span, pattern, body } = self;

        lower_telescope_inst(&pattern.params, ctx, |ctx, params| {
            let (_, uri) = ctx.symbol_table.lookup(&pattern.name)?;
            let name = IdBound {
                span: Some(pattern.name.span),
                id: pattern.name.id.clone(),
                uri: uri.clone(),
            };
            Ok(ast::Case {
                span: Some(*span),
                pattern: ast::Pattern { is_copattern: false, name, params },
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
            let (_, uri) = ctx.symbol_table.lookup(&pattern.name)?;
            let name = ast::IdBound {
                span: Some(pattern.name.span),
                id: pattern.name.id.clone(),
                uri: uri.clone(),
            };
            Ok(ast::Case {
                span: Some(*span),
                pattern: ast::Pattern { is_copattern: true, name, params },
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
        if let Some(idx) = ctx.lookup_local(name) {
            let name = VarBound { span: Some(name.span), id: name.id.clone() };
            return Ok(ast::Exp::Variable(Variable {
                span: Some(*span),
                idx,
                name,
                inferred_type: None,
            }));
        }

        // If we find the identifier in the global context then we have to lower
        // it to a call or a type constructor.
        let (meta, uri) = ctx.symbol_table.lookup(name)?;
        match meta {
            DeclMeta::Data { params, .. } | DeclMeta::Codata { params, .. } => {
                let name = IdBound { span: Some(name.span), id: name.id.clone(), uri: uri.clone() };
                Ok(ast::Exp::TypCtor(ast::TypCtor {
                    span: Some(*span),
                    name,
                    args: lower_args(*span, args, params.clone(), ctx)?,
                }))
            }
            DeclMeta::Def { .. } | DeclMeta::Dtor { .. } => {
                Err(LoweringError::MustUseAsDotCall { name: name.clone(), span: span.to_miette() })
            }
            DeclMeta::Ctor { params, .. } => {
                let name = IdBound { span: Some(name.span), id: name.id.clone(), uri: uri.clone() };
                Ok(ast::Exp::Call(ast::Call {
                    span: Some(*span),
                    kind: ast::CallKind::Constructor,
                    name,
                    args: lower_args(*span, args, params.clone(), ctx)?,
                    inferred_type: None,
                }))
            }
            DeclMeta::Codef { params, .. } => {
                let name = IdBound { span: Some(name.span), id: name.id.clone(), uri: uri.clone() };
                Ok(ast::Exp::Call(ast::Call {
                    span: Some(*span),
                    kind: ast::CallKind::Codefinition,
                    name,
                    args: lower_args(*span, args, params.clone(), ctx)?,
                    inferred_type: None,
                }))
            }
            DeclMeta::Let { params, .. } => {
                let name = IdBound { span: Some(name.span), id: name.id.clone(), uri: uri.clone() };
                Ok(ast::Exp::Call(ast::Call {
                    span: Some(*span),
                    kind: ast::CallKind::LetBound,
                    name,
                    args: lower_args(*span, args, params.clone(), ctx)?,
                    inferred_type: None,
                }))
            }
        }
    }
}

impl Lower for cst::exp::DotCall {
    type Target = ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::DotCall { span, exp, name, args } = self;

        let (meta, uri) = ctx.symbol_table.lookup(name)?;
        let (meta, uri) = (meta.clone(), uri.clone());

        match meta {
            DeclMeta::Dtor { params, .. } => Ok(ast::Exp::DotCall(ast::DotCall {
                span: Some(*span),
                kind: ast::DotCallKind::Destructor,
                exp: exp.lower(ctx)?,
                name: IdBound { span: Some(name.span), id: name.id.clone(), uri },
                args: lower_args(*span, args, params, ctx)?,
                inferred_type: None,
            })),
            DeclMeta::Def { params, .. } => Ok(ast::Exp::DotCall(ast::DotCall {
                span: Some(*span),
                kind: ast::DotCallKind::Definition,
                exp: exp.lower(ctx)?,
                name: IdBound { span: Some(name.span), id: name.id.clone(), uri },
                args: lower_args(*span, args, params, ctx)?,
                inferred_type: None,
            })),
            _ => Err(LoweringError::CannotUseAsDotCall {
                name: name.clone(),
                span: span.to_miette(),
            }),
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

impl Lower for cst::exp::HoleKind {
    type Target = ast::MetaVarKind;

    fn lower(&self, _ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        match self {
            cst::exp::HoleKind::MustSolve => Ok(ast::MetaVarKind::MustSolve),
            cst::exp::HoleKind::CanSolve => Ok(ast::MetaVarKind::CanSolve),
        }
    }
}

impl Lower for cst::exp::Hole {
    type Target = ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Hole { span, kind, .. } = self;
        let kind = kind.lower(ctx)?;
        let mv = ctx.fresh_metavar(Some(*span), kind);
        let args = ctx.subst_from_ctx();
        Ok(Hole {
            span: Some(*span),
            kind,
            metavar: mv,
            inferred_type: None,
            inferred_ctx: None,
            args,
            solution: None,
        }
        .into())
    }
}

impl Lower for cst::exp::NatLit {
    type Target = ast::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::NatLit { span, val } = self;

        // We have to check whether "Z" is declared as a constructor or codefinition.
        // We assume that if Z exists, then S exists as well and is of the same kind.
        let (z_kind, uri) = ctx
            .symbol_table
            .lookup(&Ident { span: *span, id: "Z".to_string() })
            .map_err(|_| LoweringError::NatLiteralCannotBeDesugared { span: span.to_miette() })?;
        let call_kind = match z_kind {
            DeclMeta::Codef { .. } => ast::CallKind::Codefinition,
            DeclMeta::Ctor { .. } => ast::CallKind::Constructor,
            _ => return Err(LoweringError::NatLiteralCannotBeDesugared { span: span.to_miette() }),
        };

        let mut out = ast::Exp::Call(ast::Call {
            span: Some(*span),
            kind: call_kind,
            name: ast::IdBound { span: Some(*span), id: "Z".to_owned(), uri: uri.clone() },
            args: ast::Args { args: vec![] },
            inferred_type: None,
        });

        let mut i = BigUint::from(0usize);

        while &i != val {
            i += 1usize;
            out = ast::Exp::Call(ast::Call {
                span: Some(*span),
                kind: call_kind,
                name: ast::IdBound { span: Some(*span), id: "S".to_owned(), uri: uri.clone() },
                args: ast::Args {
                    args: vec![ast::Arg::UnnamedArg { arg: Box::new(out), erased: false }],
                },
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
        let (_, uri) = ctx.symbol_table.lookup(&Ident { span: *span, id: "Fun".to_owned() })?;
        Ok(ast::TypCtor {
            span: Some(*span),
            name: ast::IdBound { span: Some(*span), id: "Fun".to_owned(), uri: uri.clone() },
            args: ast::Args {
                args: vec![
                    ast::Arg::UnnamedArg { arg: from.lower(ctx)?, erased: false },
                    ast::Arg::UnnamedArg { arg: to.lower(ctx)?, erased: false },
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
                name: Ident { span: *span, id: "ap".to_owned() },
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
        BindingSite::Wildcard { span } => Ident { span: *span, id: "_".to_owned() },
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
                name: ast::VarBind { span: Some(bs_to_span(param)), id: bs_to_name(param).id },
                typ: None,
                erased: false,
            },
            ret_typ: ctx.bind_single(param, |ctx| ret_typ.lower(ctx))?,
        })
    }
}

#[cfg(test)]
mod lower_args_tests {
    use url::Url;

    use parser::cst::decls::Telescope;

    use crate::symbol_table::SymbolTable;

    use super::*;

    #[test]
    fn test_empty() {
        let given = vec![];
        let expected = Telescope(vec![]);
        let mut ctx =
            Ctx::empty(Url::parse("inmemory:///scratch.pol").unwrap(), SymbolTable::default());
        let res = lower_args(Span::default(), &given, expected, &mut ctx);
        assert_eq!(res.unwrap(), ast::Args { args: vec![] })
    }
}
