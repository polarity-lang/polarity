use crate::ir;
use crate::result::BackendError;

use super::traits::ToIR;

impl ToIR for ast::Module {
    type Target = ir::Module;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let ast::Module { uri, use_decls, decls, meta_vars: _ } = self;

        let mut def_decls = Vec::new();
        let mut codef_decls = Vec::new();
        let mut let_decls = Vec::new();

        for decl in decls {
            match decl {
                ast::Decl::Def(def) => def_decls.push(def.to_ir()?),
                ast::Decl::Codef(codef) => codef_decls.push(codef.to_ir()?),
                ast::Decl::Let(tl_let) => let_decls.push(tl_let.to_ir()?),
                ast::Decl::Data(_) => {}
                ast::Decl::Codata(_) => {}
                ast::Decl::Infix(_) => {}
            }
        }

        Ok(ir::Module {
            uri: uri.clone(),
            use_decls: use_decls.clone(),
            def_decls,
            codef_decls,
            let_decls,
        })
    }
}

impl ToIR for ast::Def {
    type Target = ir::Def;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let ast::Def { name, params, cases, .. } = self;

        let params = params.to_ir()?;
        let cases =
            cases.iter().flat_map(|c| c.to_ir().transpose()).collect::<Result<Vec<_>, _>>()?;

        Ok(ir::Def { name: name.to_string(), params, cases })
    }
}

impl ToIR for ast::Codef {
    type Target = ir::Codef;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let ast::Codef { name, params, cases, .. } = self;

        let params = params.to_ir()?;
        let cases =
            cases.iter().flat_map(|c| c.to_ir().transpose()).collect::<Result<Vec<_>, _>>()?;

        Ok(ir::Codef { name: name.to_string(), params, cases })
    }
}

impl ToIR for ast::Let {
    type Target = ir::Let;

    fn to_ir(&self) -> Result<Self::Target, BackendError> {
        let ast::Let { name, params, body, .. } = self;

        let params = params.to_ir()?;
        let body = Box::new(body.to_ir()?);

        Ok(ir::Let { name: name.to_string(), params, body })
    }
}
