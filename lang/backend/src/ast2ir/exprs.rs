use ast::LocalComatch;

use crate::ir;
use crate::result::BackendError;

use super::traits::ToIR;

impl ToIR for ast::Exp {
    type Target = ir::Exp;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let out = match self {
            ast::Exp::Variable(variable) => ir::Exp::Variable(variable.to_ir()?),
            ast::Exp::TypCtor(typ_ctor) => typ_ctor.to_ir()?,
            ast::Exp::Call(call) => call.to_ir()?,
            ast::Exp::DotCall(dot_call) => dot_call.to_ir()?,
            ast::Exp::Anno(anno) => anno.to_ir()?,
            ast::Exp::TypeUniv(type_univ) => type_univ.to_ir()?,
            ast::Exp::LocalMatch(local_match) => ir::Exp::DefCall(local_match.to_ir()?),
            ast::Exp::LocalComatch(local_comatch) => ir::Exp::LocalComatch(local_comatch.to_ir()?),
            ast::Exp::Hole(hole) => hole.to_ir()?,
        };

        Ok(out)
    }
}

impl ToIR for ast::Variable {
    type Target = ir::Variable;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let ast::Variable { name, .. } = self;

        Ok(ir::Variable { name: name.to_string() })
    }
}

impl ToIR for ast::TypCtor {
    type Target = ir::Exp;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        // Type constructors have no runtime relevance and is hence replaced by a zero-sized term.
        Ok(ir::Exp::ZST)
    }
}

impl ToIR for ast::Call {
    type Target = ir::Exp;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let ast::Call { kind, name, args, .. } = self;

        let args = args.to_ir()?;

        let call = ir::Call { name: name.to_string(), module_uri: name.uri.clone(), args };

        Ok(match kind {
            ast::CallKind::Constructor => ir::Exp::CtorCall(call),
            ast::CallKind::Codefinition => ir::Exp::CodefCall(call),
            ast::CallKind::LetBound => ir::Exp::LetCall(call),
        })
    }
}

impl ToIR for ast::DotCall {
    type Target = ir::Exp;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let ast::DotCall { kind, exp, name, args, .. } = self;

        let args = args.to_ir()?;
        let exp = Box::new(exp.to_ir()?);

        let dot_call =
            ir::DotCall { exp, module_uri: name.uri.clone(), name: name.to_string(), args };

        Ok(match kind {
            ast::DotCallKind::Destructor => ir::Exp::DtorCall(dot_call),
            ast::DotCallKind::Definition => ir::Exp::DefCall(dot_call),
        })
    }
}

impl ToIR for ast::Anno {
    type Target = ir::Exp;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let ast::Anno { exp, .. } = self;
        // For type annotations `e: t`, we throw away the type `t` and convert `e` to IR.
        exp.to_ir()
    }
}

impl ToIR for ast::TypeUniv {
    type Target = ir::Exp;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        // The universe has no runtime relevance and is hence replaced by a zero-sized term.
        Ok(ir::Exp::ZST)
    }
}

impl ToIR for ast::LocalMatch {
    type Target = ir::DotCall;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let ast::LocalMatch { on_exp, cases, .. } = self;

        let on_exp = Box::new(on_exp.to_ir()?);
        let ast::Cases::Checked{cases:_, args, lifted_def} = cases else {return Err(BackendError::Impossible("Encountered unchecked local match".to_owned()))};
        Ok(ir::DotCall{
            exp : on_exp,
            name : lifted_def.id.clone(),
            module_uri: lifted_def.uri.clone(),
            args : args.to_ir()?
        })
    }
}

impl ToIR for ast::LocalComatch {
    type Target = ir::LocalComatch;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let LocalComatch { cases, .. } = self;

        let cases = cases.to_ir()?;

        Ok(ir::LocalComatch { cases })
    }
}

impl ToIR for ast::Hole {
    type Target = ir::Exp;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let ast::Hole { kind, solution, .. } = self;

        let res =
            match kind {
                ast::MetaVarKind::MustSolve | ast::MetaVarKind::Inserted => match solution {
                    Some(solution) => solution.to_ir()?,
                    None => return Err(BackendError::Impossible(
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

impl ToIR for ast::Case {
    type Target = ir::Case;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let ast::Case { pattern, body, .. } = self;
        let ast::Pattern { span: _, is_copattern, params, name } = pattern;

        let params = params.to_ir()?;

        let pattern = ir::Pattern {
            is_copattern: *is_copattern,
            name: name.to_string(),
            module_uri: name.uri.clone(),
            params,
        };

        let body = match body {
            Some(body) => Some(Box::new(body.to_ir()?)),
            None => None,
        };

        Ok(ir::Case { pattern, body })
    }
}

impl ToIR for ast::Args {
    type Target = Vec<ir::Exp>;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let ast::Args { args, .. } = self;

        args.iter()
            .filter(|arg| !arg.erased())
            .map(|arg| arg.exp())
            .map(|arg| arg.to_ir())
            .collect()
    }
}

impl ToIR for ast::Telescope {
    type Target = Vec<String>;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let ast::Telescope { params, .. } = self;

        Ok(params
            .iter()
            .filter(|param| !param.erased)
            .map(|param| param.name.to_string())
            .collect())
    }
}

impl ToIR for ast::TelescopeInst {
    type Target = Vec<String>;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let ast::TelescopeInst { params, .. } = self;

        Ok(params
            .iter()
            .filter(|param| !param.erased)
            .map(|param| param.name.to_string())
            .collect())
    }
}
