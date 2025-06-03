use crate::ir;
use crate::result::BackendResult;

use super::traits::ToIR;

impl ToIR for polarity_lang_ast::Module {
    type Target = ir::Module;

    fn to_ir(&self) -> BackendResult<Self::Target> {
        let polarity_lang_ast::Module { uri, use_decls, decls, meta_vars: _ } = self;

        let mut def_decls = Vec::new();
        let mut codef_decls = Vec::new();
        let mut let_decls = Vec::new();

        for decl in decls {
            match decl {
                polarity_lang_ast::Decl::Def(def) => def_decls.push(def.to_ir()?),
                polarity_lang_ast::Decl::Codef(codef) => codef_decls.push(codef.to_ir()?),
                polarity_lang_ast::Decl::Let(tl_let) => let_decls.push(tl_let.to_ir()?),
                polarity_lang_ast::Decl::Extern(_) => {}
                polarity_lang_ast::Decl::Data(_) => {}
                polarity_lang_ast::Decl::Codata(_) => {}
                polarity_lang_ast::Decl::Infix(_) => {}
                polarity_lang_ast::Decl::Note(_) => {}
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

impl ToIR for polarity_lang_ast::Def {
    type Target = ir::Def;

    fn to_ir(&self) -> BackendResult<Self::Target> {
        let polarity_lang_ast::Def { name, params, cases, .. } = self;

        let params = params.to_ir()?;
        let cases =
            cases.iter().flat_map(|c| c.to_ir().transpose()).collect::<Result<Vec<_>, _>>()?;

        Ok(ir::Def { name: name.to_string(), params, cases })
    }
}

impl ToIR for polarity_lang_ast::Codef {
    type Target = ir::Codef;

    fn to_ir(&self) -> BackendResult<Self::Target> {
        let polarity_lang_ast::Codef { name, params, cases, .. } = self;

        let params = params.to_ir()?;
        let cases =
            cases.iter().flat_map(|c| c.to_ir().transpose()).collect::<Result<Vec<_>, _>>()?;

        Ok(ir::Codef { name: name.to_string(), params, cases })
    }
}

impl ToIR for polarity_lang_ast::Let {
    type Target = ir::Let;

    fn to_ir(&self) -> BackendResult<Self::Target> {
        let polarity_lang_ast::Let { name, params, body, .. } = self;

        let params = params.to_ir()?;
        let body = Box::new(body.to_ir()?);

        Ok(ir::Let { name: name.to_string(), params, body })
    }
}
