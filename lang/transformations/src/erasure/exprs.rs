use super::ctx::Ctx;
use super::result::ErasureError;
use super::traits::Erasure;

impl Erasure for ast::Exp {
    fn erase(&mut self, ctx: &Ctx) -> Result<(), ErasureError> {
        match self {
            ast::Exp::Variable(variable) => {
                variable.erase(ctx)?;
            }
            ast::Exp::TypCtor(_) => {
                return Err(ErasureError::Impossible(
                    "Encountered unexpected type constructor".to_owned(),
                ));
            }
            ast::Exp::Call(call) => {
                call.erase(ctx)?;
            }
            ast::Exp::DotCall(dot_call) => {
                dot_call.erase(ctx)?;
            }
            ast::Exp::Anno(anno) => {
                anno.exp.erase(ctx)?;
            }
            ast::Exp::TypeUniv(_) => {
                return Err(ErasureError::Impossible(
                    "Encountered unexpected type universe".to_owned(),
                ));
            }
            ast::Exp::LocalMatch(local_match) => {
                local_match.erase(ctx)?;
            }
            ast::Exp::LocalComatch(local_comatch) => {
                local_comatch.erase(ctx)?;
            }
            ast::Exp::Hole(hole) => {
                hole.erase(ctx)?;
            }
        }
        Ok(())
    }
}

impl Erasure for ast::Variable {
    fn erase(&mut self, _ctx: &Ctx) -> Result<(), ErasureError> {
        Ok(())
    }
}

impl Erasure for ast::Call {
    fn erase(&mut self, ctx: &Ctx) -> Result<(), ErasureError> {
        for arg in &mut self.args.args {
            arg.erase(ctx)?;
        }
        Ok(())
    }
}

impl Erasure for ast::DotCall {
    fn erase(&mut self, ctx: &Ctx) -> Result<(), ErasureError> {
        self.exp.erase(ctx)?;
        for arg in &mut self.args.args {
            arg.erase(ctx)?;
        }
        Ok(())
    }
}

impl Erasure for ast::LocalMatch {
    fn erase(&mut self, ctx: &Ctx) -> Result<(), ErasureError> {
        self.on_exp.erase(ctx)?;
        for case in &mut self.cases {
            case.erase(ctx)?;
        }
        Ok(())
    }
}

impl Erasure for ast::LocalComatch {
    fn erase(&mut self, ctx: &Ctx) -> Result<(), ErasureError> {
        for case in &mut self.cases {
            case.erase(ctx)?;
        }
        Ok(())
    }
}

impl Erasure for ast::Hole {
    fn erase(&mut self, ctx: &Ctx) -> Result<(), ErasureError> {
        match self.kind {
            // If the hole must have been solved during elaboration, ensure we have a solution; erase the solution as well.
            ast::MetaVarKind::MustSolve | ast::MetaVarKind::Inserted => {
                if let Some(solution) = &mut self.solution {
                    solution.erase(ctx)?;
                } else {
                    return Err(ErasureError::Impossible(
                        "Encountered hole without solution that must be solved during typechecking"
                            .to_owned(),
                    ));
                }
            }
            ast::MetaVarKind::CanSolve => {}
        }
        Ok(())
    }
}
impl Erasure for ast::Case {
    fn erase(&mut self, ctx: &Ctx) -> Result<(), ErasureError> {
        let params_erased = if self.pattern.is_copattern {
            &ctx.symbol_table.lookup_dtor(&self.pattern.name.uri, &self.pattern.name.id)?.params
        } else {
            &ctx.symbol_table.lookup_ctor(&self.pattern.name.uri, &self.pattern.name.id)?.params
        }
        .clone();

        for (param, params_erased) in
            self.pattern.params.params.iter_mut().zip(params_erased.iter())
        {
            param.erased = params_erased.erased;
        }

        if let Some(body) = &mut self.body {
            body.erase(ctx)?;
        }
        Ok(())
    }
}

impl Erasure for ast::Arg {
    fn erase(&mut self, ctx: &Ctx) -> Result<(), ErasureError> {
        match self {
            ast::Arg::UnnamedArg { arg, erased } => {
                *erased = true;
                arg.erase(ctx)?;
            }
            ast::Arg::NamedArg { name: _, arg, erased } => {
                *erased = true;
                arg.erase(ctx)?;
            }
            ast::Arg::InsertedImplicitArg { hole, erased } => {
                *erased = true;
                hole.erase(ctx)?;
            }
        }
        Ok(())
    }
}
