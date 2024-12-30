use ast::LocalComatch;

use crate::ir;

use super::ctx::Ctx;
use super::result::ErasureError;
use super::traits::Erasure;

impl Erasure for ast::Exp {
    type Target = ir::Exp;

    fn erase(&self, ctx: &mut Ctx) -> Result<Self::Target, ErasureError> {
        let out = match self {
            ast::Exp::Variable(variable) => ir::Exp::Variable(variable.erase(ctx)?),
            ast::Exp::TypCtor(_) => {
                return Err(ErasureError::Impossible(
                    "Encountered unexpected type constructor".to_owned(),
                ))
            }
            ast::Exp::Call(call) => match call.kind {
                ast::CallKind::Constructor => ir::Exp::CtorCall(call.erase(ctx)?),
                ast::CallKind::Codefinition => ir::Exp::CodefCall(call.erase(ctx)?),
                ast::CallKind::LetBound => ir::Exp::LetCall(call.erase(ctx)?),
            },
            ast::Exp::DotCall(dot_call) => match dot_call.kind {
                ast::DotCallKind::Destructor => ir::Exp::DtorCall(dot_call.erase(ctx)?),
                ast::DotCallKind::Definition => ir::Exp::DefCall(dot_call.erase(ctx)?),
            },
            ast::Exp::Anno(anno) => anno.exp.erase(ctx)?,
            ast::Exp::TypeUniv(_) => {
                return Err(ErasureError::Impossible(
                    "Encountered unexpected type universe".to_owned(),
                ))
            }
            ast::Exp::LocalMatch(local_match) => ir::Exp::LocalMatch(local_match.erase(ctx)?),
            ast::Exp::LocalComatch(local_comatch) => {
                ir::Exp::LocalComatch(local_comatch.erase(ctx)?)
            }
            ast::Exp::Hole(hole) => hole.erase(ctx)?,
        };

        Ok(out)
    }
}

impl Erasure for ast::Variable {
    type Target = ir::Variable;

    fn erase(&self, ctx: &mut Ctx) -> Result<Self::Target, ErasureError> {
        let ast::Variable { idx, name, .. } = self;

        if ctx.is_erased(*idx) {
            return Err(ErasureError::Impossible("Encountered erased variable".to_owned()));
        }

        Ok(ir::Variable { name: name.to_string() })
    }
}

impl Erasure for ast::Call {
    type Target = ir::Call;

    fn erase(&self, ctx: &mut Ctx) -> Result<Self::Target, ErasureError> {
        let ast::Call { kind, name, args, .. } = self;

        let params = match kind {
            ast::CallKind::Constructor => {
                &ctx.symbol_table.lookup_ctor(&name.uri, &name.id)?.params
            }
            ast::CallKind::Codefinition => {
                &ctx.symbol_table.lookup_codef(&name.uri, &name.id)?.params
            }
            ast::CallKind::LetBound => &ctx.symbol_table.lookup_let(&name.uri, &name.id)?.params,
        };

        let args = params
            .iter()
            .zip(args.args.iter())
            .filter(|(param, _)| !param.erased)
            .map(|(_, arg)| arg.exp())
            .collect::<Vec<_>>();

        let args = args.into_iter().map(|arg| arg.erase(ctx)).collect::<Result<_, _>>()?;

        Ok(ir::Call { name: name.to_string(), module_uri: name.uri.clone(), args })
    }
}

impl Erasure for ast::DotCall {
    type Target = ir::DotCall;

    fn erase(&self, ctx: &mut Ctx) -> Result<Self::Target, ErasureError> {
        let ast::DotCall { kind, exp, name, args, .. } = self;

        let params = match kind {
            ast::DotCallKind::Destructor => {
                &ctx.symbol_table.lookup_dtor(&name.uri, &name.id)?.params
            }
            ast::DotCallKind::Definition => {
                &ctx.symbol_table.lookup_def(&name.uri, &name.id)?.params
            }
        };

        let args = params
            .iter()
            .zip(args.args.iter())
            .filter(|(param, _)| !param.erased)
            .map(|(_, arg)| arg.exp())
            .collect::<Vec<_>>();

        let exp = Box::new(exp.erase(ctx)?);
        let args = args.into_iter().map(|arg| arg.erase(ctx)).collect::<Result<_, _>>()?;

        Ok(ir::DotCall { exp, module_uri: name.uri.clone(), name: name.to_string(), args })
    }
}

impl Erasure for ast::LocalMatch {
    type Target = ir::LocalMatch;

    fn erase(&self, ctx: &mut Ctx) -> Result<Self::Target, ErasureError> {
        let ast::LocalMatch { on_exp, cases, .. } = self;

        let on_exp = Box::new(on_exp.erase(ctx)?);
        let cases = cases.erase(ctx)?;

        Ok(ir::LocalMatch { on_exp, cases })
    }
}

impl Erasure for ast::LocalComatch {
    type Target = ir::LocalComatch;

    fn erase(&self, ctx: &mut Ctx) -> Result<Self::Target, ErasureError> {
        let LocalComatch { cases, .. } = self;

        let cases = cases.erase(ctx)?;

        Ok(ir::LocalComatch { cases })
    }
}

impl Erasure for ast::Hole {
    type Target = ir::Exp;

    fn erase(&self, ctx: &mut Ctx) -> Result<Self::Target, ErasureError> {
        let ast::Hole { kind, solution, .. } = self;

        let res =
            match kind {
                ast::MetaVarKind::MustSolve | ast::MetaVarKind::Inserted => match solution {
                    Some(solution) => solution.erase(ctx)?,
                    None => return Err(ErasureError::Impossible(
                        "Encountered hole without solution that must be solved during typechecking"
                            .to_owned(),
                    )),
                },
                ast::MetaVarKind::CanSolve => {
                    ir::Exp::Panic(ir::Panic { message: "not yet implemented".to_owned() })
                }
            };

        Ok(res)
    }
}

impl Erasure for ast::Case {
    type Target = ir::Case;

    fn erase(&self, ctx: &mut Ctx) -> Result<Self::Target, ErasureError> {
        let ast::Case { pattern, body, .. } = self;
        let ast::Pattern { is_copattern, params: params_inst, name } = pattern;

        let params = if *is_copattern {
            &ctx.symbol_table.lookup_dtor(&name.uri, &name.id)?.params
        } else {
            &ctx.symbol_table.lookup_ctor(&name.uri, &name.id)?.params
        }
        .clone();

        let params_erased = params
            .iter()
            .zip(params_inst.params.iter())
            .filter(|(param, _)| !param.erased)
            .map(|(_, param)| param.name.to_string())
            .collect();

        let pattern = ir::Pattern {
            is_copattern: *is_copattern,
            name: name.to_string(),
            module_uri: name.uri.clone(),
            params: params_erased,
        };

        let body = match body {
            Some(body) => Some(Box::new(ctx.bind(&params, |ctx| body.erase(ctx))?)),
            None => None,
        };

        Ok(ir::Case { pattern, body })
    }
}
