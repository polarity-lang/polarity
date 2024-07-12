use codespan::Span;
use lifting::LiftResult;
use printer::PrintInCtx;
use renaming::Rename;

use syntax::ast::*;
use xfunc::matrix;
use xfunc::result::XfuncError;

use super::{DatabaseView, Edit};

pub struct Xfunc {
    pub title: String,
    pub edits: Vec<Edit>,
}

impl<'a> DatabaseView<'a> {
    pub fn xfunc(&self, type_name: &str) -> Result<Xfunc, crate::Error> {
        let prg = self.tst()?;

        let decl_spans =
            prg.map.values().map(|decl| (decl.name().clone(), decl.span().unwrap())).collect();

        // xdefs and xtors before xfunc
        let xdefs = prg.xdefs_for_type(type_name);
        let xtors = prg.xtors_for_type(type_name);

        // Filter out dirty declarations of the type being xfunctionalized which are handled separately
        let mut filter_out = HashSet::default();
        filter_out.extend(xdefs.clone());
        filter_out.extend(xtors);

        let LiftResult { prg, modified_decls: mut dirty_decls, .. } = lifting::lift(prg, type_name);
        dirty_decls.retain(|name| !filter_out.contains(name));

        let mat = xfunc::as_matrix(&prg)?;

        let type_span =
            mat.map.get(type_name).and_then(|x| x.span).ok_or(XfuncError::Impossible {
                message: format!("Could not resolve {type_name}"),
                span: None,
            })?;

        let original = Original { type_span, decl_spans, xdefs };

        let repr = xfunc::repr(&mat, type_name)?;

        let result = match repr {
            xfunc::matrix::Repr::Data => refunctionalize(prg, &mat, type_name),
            xfunc::matrix::Repr::Codata => defunctionalize(prg, &mat, type_name),
        }?;

        Ok(generate_edits(original, dirty_decls, result))
    }
}

struct Original {
    xdefs: Vec<Ident>,
    type_span: Span,
    decl_spans: HashMap<Ident, Span>,
}

struct XfuncResult {
    title: String,
    decls: Module,
    /// The new type (co)data definition as well as all associated (co)definitions
    new_decls: Vec<Decl>,
}

fn generate_edits(original: Original, dirty_decls: HashSet<Ident>, result: XfuncResult) -> Xfunc {
    let XfuncResult { title, decls, new_decls } = result;

    // Edits for the type that has been xfunctionalized
    // Here we rewrite the entire (co)data declaration and its associated (co)definitions
    let new_items = Items { items: new_decls };
    let type_text = new_items.print_to_string_in_ctx(&decls);

    let mut edits = vec![Edit { span: original.type_span, text: type_text }];

    // Edits for all other declarations that have been touched
    // Here we surgically rewrite only the declarations that have been changed
    for name in dirty_decls {
        let decl = &decls.map[&name];
        let decl = decl.clone().rename();
        let span = original.decl_spans[&name];
        let text = decl.print_to_string_in_ctx(&decls);
        edits.push(Edit { span, text });
    }

    // Remove all top-level definitions of the previous decomposition
    for name in original.xdefs {
        let span = original.decl_spans[&name];
        edits.push(Edit { span, text: "".to_owned() });
    }

    Xfunc { title, edits }
}

fn refunctionalize(
    prg: Module,
    mat: &matrix::Prg,
    type_name: &str,
) -> Result<XfuncResult, crate::Error> {
    let (codata, dtors, codefs) = xfunc::as_codata(mat, type_name)?;

    let mut decls = prg;
    let map = &mut decls.map;
    map.insert(codata.name.clone(), Decl::Codata(codata.clone()));
    map.extend(codefs.clone().into_iter().map(|def| (def.name.clone(), Decl::Codef(def))));
    map.extend(dtors.into_iter().map(|dtor| (dtor.name.clone(), Decl::Dtor(dtor))));

    let codata = codata.rename();
    let decls = decls.rename();
    let codefs = codefs.into_iter().map(Rename::rename);

    // FIXME: Unnecessary duplication
    let mut new_decls = vec![Decl::Codata(codata)];
    new_decls.extend(codefs.map(Decl::Codef));

    Ok(XfuncResult { title: format!("Refunctionalize {type_name}"), decls, new_decls })
}

fn defunctionalize(
    prg: Module,
    mat: &matrix::Prg,
    type_name: &str,
) -> Result<XfuncResult, crate::Error> {
    let (data, ctors, defs) = xfunc::as_data(mat, type_name)?;

    let mut decls = prg;
    let map = &mut decls.map;

    map.insert(data.name.clone(), Decl::Data(data.clone()));
    map.extend(defs.clone().into_iter().map(|def| (def.name.clone(), Decl::Def(def))));
    map.extend(ctors.into_iter().map(|ctor| (ctor.name.clone(), Decl::Ctor(ctor))));

    let data = data.rename();
    let decls = decls.rename();
    let defs = defs.into_iter().map(Rename::rename);

    // FIXME: Unnecessary duplication
    let mut new_decls = vec![Decl::Data(data)];
    new_decls.extend(defs.into_iter().map(Decl::Def));

    Ok(XfuncResult { title: format!("Defunctionalize {type_name}"), decls, new_decls })
}
