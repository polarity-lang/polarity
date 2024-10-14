use codespan::Span;
use printer::Print;
use transformations::LiftResult;
use transformations::Rename;

use ast::*;
use lowering::DeclMeta;
use parser::cst;
use transformations::matrix;
use transformations::result::XfuncError;
use url::Url;

use crate::database::Database;

use super::Edit;

pub struct Xfunc {
    pub title: String,
    pub edits: Vec<Edit>,
}

impl Database {
    pub fn all_type_names(&mut self, uri: &Url) -> Result<Vec<cst::Ident>, crate::Error> {
        let symbol_table = self.symbol_table(uri)?;
        Ok(symbol_table
            .iter()
            .filter(|(_, decl_meta)| {
                matches!(decl_meta, DeclMeta::Data { .. } | DeclMeta::Codata { .. })
            })
            .map(|(name, _)| name.clone())
            .collect())
    }

    pub fn xfunc(&mut self, uri: &Url, type_name: &str) -> Result<Xfunc, crate::Error> {
        let module = self.load_module(uri)?;

        let decl_spans =
            module.decls.iter().map(|decl| (decl.name().clone(), decl.span().unwrap())).collect();

        // xdefs and xtors before xfunc
        let xdefs = module.xdefs_for_type(type_name);
        let xtors = module.xtors_for_type(type_name);

        // Filter out dirty declarations of the type being xfunctionalized which are handled separately
        let mut filter_out = HashSet::default();
        filter_out.extend(xdefs.clone());
        filter_out.extend(xtors);

        let LiftResult { module, modified_decls: mut dirty_decls, .. } =
            transformations::lift(module, type_name);
        dirty_decls.retain(|name| !filter_out.contains(name));

        let mat = transformations::as_matrix(&module)?;

        let type_span = mat.map.get(&Ident::from_string(type_name)).and_then(|x| x.span).ok_or(
            XfuncError::Impossible {
                message: format!("Could not resolve {type_name}"),
                span: None,
            },
        )?;

        let original = Original { type_span, decl_spans, xdefs };

        let repr = transformations::repr(&mat, &Ident::from_string(type_name))?;

        let result = match repr {
            transformations::matrix::Repr::Data => refunctionalize(&mat, type_name),
            transformations::matrix::Repr::Codata => defunctionalize(&mat, type_name),
        }?;

        Ok(generate_edits(&module, original, dirty_decls, result))
    }
}

struct Original {
    xdefs: Vec<Ident>,
    type_span: Span,
    decl_spans: HashMap<Ident, Span>,
}

struct XfuncResult {
    title: String,
    /// The new type (co)data definition as well as all associated (co)definitions
    new_decls: Vec<Decl>,
}

fn generate_edits(
    module: &Module,
    original: Original,
    dirty_decls: HashSet<Ident>,
    result: XfuncResult,
) -> Xfunc {
    let XfuncResult { title, new_decls } = result;

    // Edits for the type that has been xfunctionalized
    // Here we rewrite the entire (co)data declaration and its associated (co)definitions
    let new_items = Module {
        uri: module.uri.clone(),
        use_decls: module.use_decls.clone(),
        decls: new_decls,
        meta_vars: module.meta_vars.clone(),
    };
    let type_text = new_items.print_to_string(None);

    let mut edits = vec![Edit { span: original.type_span, text: type_text }];

    // Edits for all other declarations that have been touched
    // Here we surgically rewrite only the declarations that have been changed
    for name in dirty_decls {
        let decl = module.lookup_decl(&name).unwrap();
        let mut decl = decl.clone();
        decl.rename();
        let span = original.decl_spans[&name];
        let text = decl.print_to_string(None);
        edits.push(Edit { span, text });
    }

    // Remove all top-level definitions of the previous decomposition
    for name in original.xdefs {
        let span = original.decl_spans[&name];
        edits.push(Edit { span, text: "".to_owned() });
    }

    Xfunc { title, edits }
}

fn refunctionalize(mat: &matrix::Prg, type_name: &str) -> Result<XfuncResult, crate::Error> {
    let (mut codata, mut codefs) = transformations::as_codata(mat, &Ident::from_string(type_name))?;

    codata.rename();
    codefs.iter_mut().for_each(|codef| codef.rename());

    let mut new_decls = vec![Decl::Codata(codata)];
    new_decls.extend(codefs.into_iter().map(Decl::Codef));

    Ok(XfuncResult { title: format!("Refunctionalize {type_name}"), new_decls })
}

fn defunctionalize(mat: &matrix::Prg, type_name: &str) -> Result<XfuncResult, crate::Error> {
    let (mut data, mut defs) = transformations::as_data(mat, &Ident::from_string(type_name))?;

    data.rename();
    defs.iter_mut().for_each(|def| def.rename());

    let mut new_decls = vec![Decl::Data(data)];
    new_decls.extend(defs.into_iter().map(Decl::Def));

    Ok(XfuncResult { title: format!("Defunctionalize {type_name}"), new_decls })
}
