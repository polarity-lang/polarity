use polarity_lang_ast::{
    VarBind,
    ctx::{BindContext, values::Binder},
};
use polarity_lang_miette_util::ToMiette;
use polarity_lang_parser::cst::{self};

use super::*;

mod codata_declaration;
mod codefinition;
mod data_declaration;
mod definition;
mod infix_declaration;
mod note_declaration;
mod toplevel_let;

// Doc Comments
//
//

impl Lower for cst::decls::DocComment {
    type Target = polarity_lang_ast::DocComment;

    fn lower(&self, _ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        Ok(polarity_lang_ast::DocComment { docs: self.docs.clone() })
    }
}

// Attributes
//
//

fn parse_attribute(s: String) -> polarity_lang_ast::Attribute {
    match s.as_str() {
        "omit_print" => polarity_lang_ast::Attribute::OmitPrint,
        "transparent" => polarity_lang_ast::Attribute::Transparent,
        "opaque" => polarity_lang_ast::Attribute::Opaque,
        v => polarity_lang_ast::Attribute::Other(v.to_string()),
    }
}
impl Lower for cst::decls::Attributes {
    type Target = polarity_lang_ast::Attributes;

    fn lower(&self, _ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        Ok(polarity_lang_ast::Attributes {
            attrs: self.attrs.clone().into_iter().map(parse_attribute).collect(),
        })
    }
}

// Use Declarations
//
//

impl Lower for cst::decls::UseDecl {
    type Target = polarity_lang_ast::UseDecl;

    fn lower(&self, _ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let cst::decls::UseDecl { span, path } = self;
        Ok(polarity_lang_ast::UseDecl { span: *span, path: path.clone() })
    }
}

// Declarations
//
//

impl Lower for cst::decls::Decl {
    type Target = polarity_lang_ast::Decl;

    fn lower(&self, ctx: &mut Ctx) -> LoweringResult<Self::Target> {
        let decl = match self {
            cst::decls::Decl::Data(data) => data.lower(ctx)?.into(),
            cst::decls::Decl::Codata(codata) => codata.lower(ctx)?.into(),
            cst::decls::Decl::Def(def) => def.lower(ctx)?.into(),
            cst::decls::Decl::Codef(codef) => codef.lower(ctx)?.into(),
            cst::decls::Decl::Let(tl_let) => tl_let.lower(ctx)?.into(),
            cst::decls::Decl::Infix(infix) => infix.lower(ctx)?.into(),
            cst::decls::Decl::Note(note) => note.lower(ctx)?.into(),

            // In the future, we plan to handle these as holes, for now this is caught in parsing.
            cst::decls::Decl::Error => {
                return Err(Box::new(LoweringError::Impossible {
                    message: "An erroneous CST must be caught in parsing stage".to_string(),
                    span: None,
                }));
            }
        };
        Ok(decl)
    }
}

// Self Parameter
//
//

fn lower_self_param<T, F: FnOnce(&mut Ctx, polarity_lang_ast::SelfParam) -> LoweringResult<T>>(
    self_param: &cst::decls::SelfParam,
    ctx: &mut Ctx,
    f: F,
) -> LoweringResult<T> {
    let cst::decls::SelfParam { span, name, typ } = self_param;
    let typ_out = typ.lower(ctx)?;
    let typ_ctor =
        typ_out.to_typctor().ok_or(LoweringError::ExpectedTypCtor { span: span.to_miette() })?;
    let name = match name {
        Some(ident) => VarBind::Var { span: Some(ident.span), id: ident.id.clone() },
        None => VarBind::Wildcard { span: None },
    };
    ctx.bind_single(name.clone(), |ctx| {
        f(ctx, polarity_lang_ast::SelfParam { info: Some(*span), name, typ: typ_ctor })
    })
}

// Telescopes
//
//

fn desugar_telescope(tel: &cst::decls::Telescope) -> cst::decls::Telescope {
    let cst::decls::Telescope(params) = tel;
    let params: Vec<cst::decls::Param> = params.iter().flat_map(desugar_param).collect();
    cst::decls::Telescope(params)
}
fn desugar_param(param: &cst::decls::Param) -> Vec<cst::decls::Param> {
    let cst::decls::Param { implicit, name, names, typ } = param;
    let mut params: Vec<cst::decls::Param> = vec![cst::decls::Param {
        implicit: *implicit,
        name: name.clone(),
        names: vec![],
        typ: typ.clone(),
    }];
    for extra_name in names {
        params.push(cst::decls::Param {
            implicit: *implicit,
            name: extra_name.clone(),
            names: vec![],
            typ: typ.clone(),
        });
    }
    params
}

/// Lower a telescope
///
/// Execute a function `f` under the context where all binders
/// of the telescope are in scope.
fn lower_telescope<T, F>(tele: &cst::decls::Telescope, ctx: &mut Ctx, f: F) -> LoweringResult<T>
where
    F: FnOnce(&mut Ctx, polarity_lang_ast::Telescope) -> LoweringResult<T>,
{
    let tel = desugar_telescope(tele);
    ctx.bind_fold_failable(
        tel.0.iter(),
        vec![],
        |ctx, params_out, param| -> LoweringResult<Binder<()>> {
            let cst::decls::Param { implicit, name, names: _, typ } = param; // The `names` field has been removed by `desugar_telescope`.
            let typ_out = typ.lower(ctx)?;
            let name = name.lower(ctx)?;
            let param_out = polarity_lang_ast::Param {
                implicit: *implicit,
                name: name.clone(),
                typ: typ_out,
                erased: false,
            };
            params_out.push(param_out);
            Ok(Binder { name, content: () })
        },
        |ctx, params| f(ctx, polarity_lang_ast::Telescope { params }),
    )?
}
