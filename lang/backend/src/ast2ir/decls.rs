use crate::ir;
use crate::result::BackendError;

use super::traits::ToIR;

impl ToIR for ast::Module {
    type Target = ir::Module;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let ast::Module { uri, use_decls, decls, meta_vars: _ } = self;

        let IRDecls { def_decls, codef_decls, let_decls } = to_ir_decls(decls)?;

        Ok(ir::Module {
            uri: uri.clone(),
            use_decls: use_decls.clone(),
            def_decls,
            codef_decls,
            let_decls,
        })
    }
}

struct IRDecls {
    def_decls: Vec<ir::Def>,
    codef_decls: Vec<ir::Codef>,
    let_decls: Vec<ir::Let>,
}

fn to_ir_decls(decls: &[ast::Decl]) -> Result<IRDecls, BackendError> {
    let mut def_decls = Vec::new();
    let mut codef_decls = Vec::new();
    let mut let_decls = Vec::new();

    for decl in decls {
        match decl {
            ast::Decl::Def(def) => {
                if let Some(def) = def.to_ir()? {
                    def_decls.push(def);
                }
            }
            ast::Decl::Codef(codef) => codef_decls.push(codef.to_ir()?),
            ast::Decl::Let(tl_let) => {
                if let Some(tl_let) = tl_let.to_ir()? {
                    let_decls.push(tl_let);
                }
            }
            ast::Decl::Data(_) => {}
            ast::Decl::Codata(_) => {}
        }
    }

    Ok(IRDecls { def_decls, codef_decls, let_decls })
}

impl ToIR for ast::Def {
    type Target = Option<ir::Def>;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let ast::Def { name, params, self_param, cases, erased, .. } = self;

        if *erased {
            return Ok(None);
        }

        let params = params
            .params
            .iter()
            .filter(|param| !param.erased)
            .map(|param| param.name.to_string())
            .collect();

        let cases = cases.to_ir()?;

        Ok(Some(ir::Def {
            name: name.to_string(),
            self_param: self_param.name.as_ref().map(|nm| nm.to_string()).unwrap_or_default(),
            params,
            cases,
        }))
    }
}

impl ToIR for ast::Codef {
    type Target = ir::Codef;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let ast::Codef { name, params, cases, .. } = self;

        let params = params
            .params
            .iter()
            .filter(|param| !param.erased)
            .map(|param| param.name.to_string())
            .collect();

        let cases = cases.to_ir()?;

        Ok(ir::Codef { name: name.to_string(), params, cases })
    }
}

impl ToIR for ast::Let {
    type Target = Option<ir::Let>;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let ast::Let { name, params, body, erased, .. } = self;

        if *erased {
            return Ok(None);
        }

        let params = params
            .params
            .iter()
            .filter(|param| !param.erased)
            .map(|param| param.name.to_string())
            .collect();

        let body = Box::new(body.to_ir()?);

        Ok(Some(ir::Let { name: name.to_string(), params, body }))
    }
}
