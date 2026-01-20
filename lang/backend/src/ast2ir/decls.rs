use crate::ast2ir::traits::CollectToplevelNames;
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

        let mut toplevel_names = Vec::new();
        self.collect_toplevel_names(&mut toplevel_names);

        Ok(ir::Module {
            uri: uri.clone(),
            toplevel_names,
            use_decls: use_decls.clone(),
            def_decls,
            codef_decls,
            let_decls,
        })
    }
}

impl CollectToplevelNames for polarity_lang_ast::Module {
    fn collect_toplevel_names(&self, names: &mut Vec<ir::ident::Ident>) {
        let polarity_lang_ast::Module { uri: _, use_decls: _, decls, meta_vars: _ } = self;
        decls.collect_toplevel_names(names);
    }
}

impl CollectToplevelNames for polarity_lang_ast::Decl {
    fn collect_toplevel_names(&self, names: &mut Vec<ir::ident::Ident>) {
        match self {
            polarity_lang_ast::Decl::Data(data) => data.collect_toplevel_names(names),
            polarity_lang_ast::Decl::Codata(codata) => codata.collect_toplevel_names(names),
            polarity_lang_ast::Decl::Def(def) => def.collect_toplevel_names(names),
            polarity_lang_ast::Decl::Codef(codef) => codef.collect_toplevel_names(names),
            polarity_lang_ast::Decl::Let(tl_let) => tl_let.collect_toplevel_names(names),
            polarity_lang_ast::Decl::Extern(ext) => ext.collect_toplevel_names(names),
            polarity_lang_ast::Decl::Infix(_) => (),
            polarity_lang_ast::Decl::Note(_) => (),
        }
    }
}

impl ToIR for polarity_lang_ast::Def {
    type Target = ir::Def;

    fn to_ir(&self) -> BackendResult<Self::Target> {
        let polarity_lang_ast::Def { name, params, cases, .. } = self;

        let params = params.to_ir()?;
        let cases =
            cases.iter().flat_map(|c| c.to_ir().transpose()).collect::<Result<Vec<_>, _>>()?;

        Ok(ir::Def { name: name.to_string().into(), params, cases })
    }
}

impl CollectToplevelNames for polarity_lang_ast::Def {
    fn collect_toplevel_names(&self, names: &mut Vec<ir::ident::Ident>) {
        let polarity_lang_ast::Def {
            span: _,
            doc: _,
            name,
            attr: _,
            params: _,
            self_param: _,
            ret_typ: _,
            cases: _,
        } = self;
        names.push(name.to_string().into());
    }
}

impl ToIR for polarity_lang_ast::Codef {
    type Target = ir::Codef;

    fn to_ir(&self) -> BackendResult<Self::Target> {
        let polarity_lang_ast::Codef { name, params, cases, .. } = self;

        let params = params.to_ir()?;
        let cases =
            cases.iter().flat_map(|c| c.to_ir().transpose()).collect::<Result<Vec<_>, _>>()?;

        Ok(ir::Codef { name: name.to_string().into(), params, cases })
    }
}

impl CollectToplevelNames for polarity_lang_ast::Codef {
    fn collect_toplevel_names(&self, names: &mut Vec<ir::ident::Ident>) {
        let polarity_lang_ast::Codef {
            span: _,
            doc: _,
            name,
            attr: _,
            params: _,
            cases: _,
            typ: _,
        } = self;
        names.push(name.to_string().into());
    }
}

impl ToIR for polarity_lang_ast::Let {
    type Target = ir::Let;

    fn to_ir(&self) -> BackendResult<Self::Target> {
        let polarity_lang_ast::Let { name, params, body, .. } = self;

        let params = params.to_ir()?;
        let body = Box::new(body.to_ir()?);

        Ok(ir::Let { name: name.to_string().into(), params, body })
    }
}

impl CollectToplevelNames for polarity_lang_ast::Let {
    fn collect_toplevel_names(&self, names: &mut Vec<ir::ident::Ident>) {
        let polarity_lang_ast::Let { span: _, doc: _, name, attr: _, params: _, typ: _, body: _ } =
            self;
        names.push(name.to_string().into());
    }
}

impl CollectToplevelNames for polarity_lang_ast::Data {
    fn collect_toplevel_names(&self, names: &mut Vec<ir::ident::Ident>) {
        let polarity_lang_ast::Data { span: _, doc: _, name: _, attr: _, typ: _, ctors } = self;
        ctors.collect_toplevel_names(names);
    }
}

impl CollectToplevelNames for polarity_lang_ast::Ctor {
    fn collect_toplevel_names(&self, names: &mut Vec<ir::ident::Ident>) {
        let polarity_lang_ast::Ctor { span: _, doc: _, name, params: _, typ: _ } = self;
        names.push(name.to_string().into());
    }
}

impl CollectToplevelNames for polarity_lang_ast::Codata {
    fn collect_toplevel_names(&self, names: &mut Vec<ir::ident::Ident>) {
        let polarity_lang_ast::Codata { span: _, doc: _, name: _, attr: _, typ: _, dtors } = self;
        dtors.collect_toplevel_names(names);
    }
}

impl CollectToplevelNames for polarity_lang_ast::Dtor {
    fn collect_toplevel_names(&self, names: &mut Vec<ir::ident::Ident>) {
        let polarity_lang_ast::Dtor { span: _, doc: _, name, params: _, self_param: _, ret_typ: _ } =
            self;
        names.push(name.to_string().into());
    }
}

impl CollectToplevelNames for polarity_lang_ast::Extern {
    fn collect_toplevel_names(&self, names: &mut Vec<ir::ident::Ident>) {
        let polarity_lang_ast::Extern { span: _, doc: _, name, attr: _, params: _, typ: _ } = self;
        names.push(name.to_string().into());
    }
}
