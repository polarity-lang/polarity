use polarity_lang_ast::{LocalComatch, LocalLet};

use crate::ir;
use crate::result::BackendError;

use super::traits::ToIR;

impl ToIR for polarity_lang_ast::Exp {
    type Target = ir::Exp;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let out = match self {
            polarity_lang_ast::Exp::Variable(variable) => ir::Exp::Variable(variable.to_ir()?),
            polarity_lang_ast::Exp::TypCtor(typ_ctor) => typ_ctor.to_ir()?,
            polarity_lang_ast::Exp::Call(call) => call.to_ir()?,
            polarity_lang_ast::Exp::DotCall(dot_call) => dot_call.to_ir()?,
            polarity_lang_ast::Exp::Anno(anno) => anno.to_ir()?,
            polarity_lang_ast::Exp::TypeUniv(type_univ) => type_univ.to_ir()?,
            polarity_lang_ast::Exp::LocalMatch(local_match) => {
                ir::Exp::LocalMatch(local_match.to_ir()?)
            }
            polarity_lang_ast::Exp::LocalComatch(local_comatch) => {
                ir::Exp::LocalComatch(local_comatch.to_ir()?)
            }
            polarity_lang_ast::Exp::Hole(hole) => hole.to_ir()?,
            polarity_lang_ast::Exp::LocalLet(local_let) => ir::Exp::LocalLet(local_let.to_ir()?),
        };

        Ok(out)
    }
}

impl ToIR for polarity_lang_ast::Variable {
    type Target = ir::Variable;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let polarity_lang_ast::Variable { name, .. } = self;

        Ok(ir::Variable { name: name.to_string() })
    }
}

impl ToIR for polarity_lang_ast::TypCtor {
    type Target = ir::Exp;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        // Type constructors have no runtime relevance and is hence replaced by a zero-sized term.
        Ok(ir::Exp::ZST)
    }
}

impl ToIR for polarity_lang_ast::Call {
    type Target = ir::Exp;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let polarity_lang_ast::Call { kind, name, args, .. } = self;

        let args = args.to_ir()?;

        let call = ir::Call { name: name.to_string(), module_uri: name.uri.clone(), args };

        Ok(match kind {
            polarity_lang_ast::CallKind::Constructor => ir::Exp::CtorCall(call),
            polarity_lang_ast::CallKind::Codefinition => ir::Exp::CodefCall(call),
            polarity_lang_ast::CallKind::LetBound => ir::Exp::LetCall(call),
            polarity_lang_ast::CallKind::Extern => todo!(),
        })
    }
}

impl ToIR for polarity_lang_ast::DotCall {
    type Target = ir::Exp;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let polarity_lang_ast::DotCall { kind, exp, name, args, .. } = self;

        let args = args.to_ir()?;
        let exp = Box::new(exp.to_ir()?);

        let dot_call =
            ir::DotCall { exp, module_uri: name.uri.clone(), name: name.to_string(), args };

        Ok(match kind {
            polarity_lang_ast::DotCallKind::Destructor => ir::Exp::DtorCall(dot_call),
            polarity_lang_ast::DotCallKind::Definition => ir::Exp::DefCall(dot_call),
        })
    }
}

impl ToIR for polarity_lang_ast::Anno {
    type Target = ir::Exp;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let polarity_lang_ast::Anno { exp, .. } = self;
        // For type annotations `e: t`, we throw away the type `t` and convert `e` to IR.
        exp.to_ir()
    }
}

impl ToIR for polarity_lang_ast::TypeUniv {
    type Target = ir::Exp;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        // The universe has no runtime relevance and is hence replaced by a zero-sized term.
        Ok(ir::Exp::ZST)
    }
}

impl ToIR for polarity_lang_ast::LocalMatch {
    type Target = ir::LocalMatch;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let polarity_lang_ast::LocalMatch { on_exp, cases, .. } = self;

        let on_exp = Box::new(on_exp.to_ir()?);
        let cases =
            cases.iter().flat_map(|c| c.to_ir().transpose()).collect::<Result<Vec<_>, _>>()?;

        Ok(ir::LocalMatch { on_exp, cases })
    }
}

impl ToIR for polarity_lang_ast::LocalComatch {
    type Target = ir::LocalComatch;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let LocalComatch { cases, .. } = self;

        let cases =
            cases.iter().flat_map(|c| c.to_ir().transpose()).collect::<Result<Vec<_>, _>>()?;

        Ok(ir::LocalComatch { cases })
    }
}

impl ToIR for polarity_lang_ast::Hole {
    type Target = ir::Exp;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let polarity_lang_ast::Hole { kind, solution, .. } = self;

        let res =
            match kind {
                polarity_lang_ast::MetaVarKind::MustSolve
                | polarity_lang_ast::MetaVarKind::Inserted => match solution {
                    Some(solution) => solution.to_ir()?,
                    None => return Err(BackendError::Impossible(
                        "Encountered hole without solution that must be solved during typechecking"
                            .to_owned(),
                    )),
                },
                polarity_lang_ast::MetaVarKind::CanSolve => {
                    ir::Exp::Panic(ir::Panic { message: "not yet implemented".to_owned() })
                }
            };

        Ok(res)
    }
}

impl ToIR for LocalLet {
    type Target = ir::LocalLet;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let LocalLet { span: _, name, typ: _, bound, body, inferred_type: _ } = self;

        Ok(ir::LocalLet {
            name: name.to_string(),
            bound: Box::new(bound.to_ir()?),
            body: Box::new(body.to_ir()?),
        })
    }
}

impl ToIR for polarity_lang_ast::Case {
    type Target = Option<ir::Case>;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let polarity_lang_ast::Case { pattern, body, .. } = self;
        let polarity_lang_ast::Pattern { span: _, is_copattern, params, name } = pattern;

        let params = params.to_ir()?;

        let pattern = ir::Pattern {
            is_copattern: *is_copattern,
            name: name.to_string(),
            module_uri: name.uri.clone(),
            params,
        };

        let Some(body) = body else { return Ok(None) };

        Ok(Some(ir::Case { pattern, body: Box::new(body.to_ir()?) }))
    }
}

impl ToIR for polarity_lang_ast::Args {
    type Target = Vec<ir::Exp>;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let polarity_lang_ast::Args { args, .. } = self;

        args.iter()
            .filter(|arg| !arg.erased())
            .map(|arg| arg.exp())
            .map(|arg| arg.to_ir())
            .collect()
    }
}

impl ToIR for polarity_lang_ast::Telescope {
    type Target = Vec<String>;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let polarity_lang_ast::Telescope { params, .. } = self;

        Ok(params
            .iter()
            .filter(|param| !param.erased)
            .map(|param| param.name.to_string())
            .collect())
    }
}

impl ToIR for polarity_lang_ast::TelescopeInst {
    type Target = Vec<String>;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let polarity_lang_ast::TelescopeInst { params, .. } = self;

        Ok(params
            .iter()
            .filter(|param| !param.erased)
            .map(|param| param.name.to_string())
            .collect())
    }
}
