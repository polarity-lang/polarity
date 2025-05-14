use ast::{Hole, MetaVarKind, VarBound};
use miette_util::{ToMiette, codespan::Span};
use parser::cst::{self, exp::BindingSite};
use printer::Print;

use crate::{Ctx, LoweringError, LoweringResult, lower::Lower};

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
pub fn lower_args(
    span: Span,
    given: &[cst::exp::Arg],
    expected: cst::decls::Telescope,
    ctx: &mut Ctx,
) -> LoweringResult<ast::Args> {
    let mut args_out = vec![];

    // Ensure that the number of given arguments does not exceed the number of expected parameters.
    // Some expected parameters might be implicit and not require corresponding given arguments.
    if given.len() > expected.len() {
        // The unwrap is safe because in this branch there must be at least one given.
        let err = LoweringError::TooManyArgs { span: given.first().unwrap().span().to_miette() };
        return Err(err.into());
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
    ) -> LoweringResult {
        let Some(arg) = given.next() else {
            let expected = expected_bs.lower(ctx)?;
            let expected = expected.print_to_string(None);
            return Err(
                LoweringError::MissingArgForParam { expected, span: span.to_miette() }.into()
            );
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
                        }
                        .into());
                    }
                };
                if name.id != expected_name.id {
                    return Err(LoweringError::MismatchedNamedArgs {
                        given: name.to_owned(),
                        expected: expected_name.to_owned(),
                        span: exp.span().to_miette(),
                    }
                    .into());
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
                        }
                        .into());
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
        return Err(LoweringError::TooManyArgs { span: extra_arg.span().to_miette() }.into());
    }

    // All arguments have been successfully processed.
    Ok(ast::Args { args: args_out })
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
