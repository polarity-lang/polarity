use codespan::Span;
use lifting::LiftResult;
use printer::PrintToStringInCtx;
use renaming::Rename;

use data::{HashMap, HashSet};
use syntax::ast;
use syntax::common::*;
use syntax::matrix;
use syntax::ust;

use super::{DatabaseView, Edit};

pub struct Xfunc {
    pub title: String,
    pub edits: Vec<Edit>,
}

impl<'a> DatabaseView<'a> {
    pub fn xfunc(&self, type_name: &str) -> Result<Xfunc, String> {
        let prg = self.tst().map_err(|err| format!("{}", err))?;

        let decl_spans = prg
            .decls
            .map
            .values()
            .map(|decl| (decl.name().clone(), decl.info().span.unwrap()))
            .collect();

        // Filter out dirty declarations of the type being xfunctionalized which are handled separately
        let mut filter_out = HashSet::default();
        filter_out.extend(prg.decls.map[type_name].xtors().iter().cloned());
        filter_out.extend(prg.decls.map[type_name].xdefs().iter().cloned());

        let LiftResult { prg, modified_decls: mut dirty_decls } =
            lifting::lift(prg.forget(), type_name);
        dirty_decls.retain(|name| !filter_out.contains(name));

        let prg = prg.forget();
        let mat = xfunc::as_matrix(&prg);

        let type_span = mat.map[type_name].info.span.unwrap();
        let impl_span = mat.map[type_name].impl_block.as_ref().and_then(|block| block.info.span);

        let original = Original { type_span, impl_span, decl_spans };

        let result = match xfunc::repr(&mat, type_name) {
            syntax::matrix::Repr::Data => refunctionalize(prg, &mat, type_name),
            syntax::matrix::Repr::Codata => defunctionalize(prg, &mat, type_name),
        };

        Ok(generate_edits(original, dirty_decls, result))
    }
}

struct Original {
    type_span: Span,
    impl_span: Option<Span>,
    decl_spans: HashMap<Ident, Span>,
}

struct XfuncResult {
    title: String,
    decls: ust::Decls,
    new_decl: ust::Decl,
    new_impl: ust::Impl,
}

fn generate_edits(original: Original, dirty_decls: HashSet<Ident>, result: XfuncResult) -> Xfunc {
    let XfuncResult { title, decls, new_decl, new_impl } = result;

    // Edits for the type that has been xfunctionalized
    // Here we rewrite the entire (co)data declaration and its associated impl block (if any)
    let type_text = new_decl.print_to_string_in_ctx(&decls);
    let impl_text = new_impl.print_to_string_in_ctx(&decls);

    let mut edits = if let Some(impl_span) = original.impl_span {
        vec![
            Edit { span: original.type_span, text: type_text },
            Edit { span: impl_span, text: impl_text },
        ]
    } else {
        let mut type_and_impl_text = type_text;
        if !new_impl.defs.is_empty() {
            type_and_impl_text.push_str("\n\n");
            type_and_impl_text.push_str(&impl_text);
        }
        vec![Edit { span: original.type_span, text: type_and_impl_text }]
    };

    // Edits for all other declarations that have been touched
    // Here we surgically rewrite only the declarations that have been changed
    for name in dirty_decls {
        let decl = &decls.map[&name];
        let decl = decl.clone().rename();
        let span = original.decl_spans[&name];
        let text = decl.print_to_string_in_ctx(&decls);
        edits.push(Edit { span, text });
    }

    Xfunc { title, edits }
}

fn refunctionalize(prg: ust::Prg, mat: &matrix::Prg, type_name: &str) -> XfuncResult {
    let (codata, dtors, codefs) = xfunc::as_codata(mat, type_name);

    let impl_block = ust::Impl {
        info: ust::Info::empty(),
        name: type_name.to_owned(),
        defs: codefs.iter().map(|def| def.name.clone()).collect(),
    };

    let mut decls = prg.decls;
    let map = &mut decls.map;
    map.insert(codata.name.clone(), ust::Decl::Codata(codata.clone()));
    map.extend(codefs.into_iter().map(|def| (def.name.clone(), ust::Decl::Codef(def))));
    map.extend(dtors.into_iter().map(|dtor| (dtor.name.clone(), ust::Decl::Dtor(dtor))));

    let codata = codata.rename();
    let decls = decls.rename();
    let impl_block = impl_block.rename();

    XfuncResult {
        title: format!("Refunctionalize {}", type_name),
        decls,
        new_decl: ust::Decl::Codata(codata),
        new_impl: impl_block,
    }
}

fn defunctionalize(prg: ust::Prg, mat: &matrix::Prg, type_name: &str) -> XfuncResult {
    let (data, ctors, defs) = xfunc::as_data(mat, type_name);

    let impl_block = ust::Impl {
        info: ust::Info::empty(),
        name: type_name.to_owned(),
        defs: defs.iter().map(|def| def.name.clone()).collect(),
    };

    let mut decls = prg.decls;
    let map = &mut decls.map;

    map.insert(data.name.clone(), ust::Decl::Data(data.clone()));
    map.extend(defs.into_iter().map(|def| (def.name.clone(), ust::Decl::Def(def))));
    map.extend(ctors.into_iter().map(|ctor| (ctor.name.clone(), ust::Decl::Ctor(ctor))));

    let data = data.rename();
    let impl_block = impl_block.rename();
    let decls = decls.rename();

    XfuncResult {
        title: format!("Defunctionalize {}", type_name),
        decls,
        new_decl: ust::Decl::Data(data),
        new_impl: impl_block,
    }
}

trait DeclExt<P: ast::Phase> {
    fn xtors(&self) -> &[Ident];
    fn xdefs(&self) -> &[Ident];
    fn impl_block(&self) -> Option<&ast::Impl<P>>;
}

impl<P: ast::Phase> DeclExt<P> for ast::Decl<P> {
    fn xtors(&self) -> &[Ident] {
        match self {
            syntax::ast::Decl::Data(data) => &data.ctors[..],
            syntax::ast::Decl::Codata(codata) => &codata.dtors[..],
            _ => &[],
        }
    }

    fn xdefs(&self) -> &[Ident] {
        let res = match self {
            syntax::ast::Decl::Data(data) => data.impl_block.as_ref().map(|block| &block.defs[..]),
            syntax::ast::Decl::Codata(codata) => {
                codata.impl_block.as_ref().map(|block| &block.defs[..])
            }
            _ => None,
        };
        res.unwrap_or_default()
    }

    fn impl_block(&self) -> Option<&ast::Impl<P>> {
        match self {
            syntax::ast::Decl::Data(data) => data.impl_block.as_ref(),
            syntax::ast::Decl::Codata(codata) => codata.impl_block.as_ref(),
            _ => None,
        }
    }
}
