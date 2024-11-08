use ast::VarBind;
use miette_util::ToMiette;
use parser::cst::exp::BindingSite;
use parser::cst::{self};

use super::*;

mod codata_declaration;
mod codefinition;
mod data_declaration;
mod definition;
mod toplevel_let;

// Doc Comments
//
//

impl Lower for cst::decls::DocComment {
    type Target = ast::DocComment;

    fn lower(&self, _ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        Ok(ast::DocComment { docs: self.docs.clone() })
    }
}

// Attributes
//
//

fn parse_attribute(s: String) -> ast::Attribute {
    match s.as_str() {
        "omit_print" => ast::Attribute::OmitPrint,
        "transparent" => ast::Attribute::Transparent,
        "opaque" => ast::Attribute::Opaque,
        v => ast::Attribute::Other(v.to_string()),
    }
}
impl Lower for cst::decls::Attributes {
    type Target = ast::Attributes;

    fn lower(&self, _ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        Ok(ast::Attributes { attrs: self.attrs.clone().into_iter().map(parse_attribute).collect() })
    }
}

// Use Declarations
//
//

impl Lower for cst::decls::UseDecl {
    type Target = ast::UseDecl;

    fn lower(&self, _ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::decls::UseDecl { span, path } = self;
        Ok(ast::UseDecl { span: *span, path: path.clone() })
    }
}

// Declarations
//
//

impl Lower for cst::decls::Decl {
    type Target = ast::Decl;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let decl = match self {
            cst::decls::Decl::Data(data) => data.lower(ctx)?.into(),
            cst::decls::Decl::Codata(codata) => codata.lower(ctx)?.into(),
            cst::decls::Decl::Def(def) => def.lower(ctx)?.into(),
            cst::decls::Decl::Codef(codef) => codef.lower(ctx)?.into(),
            cst::decls::Decl::Let(tl_let) => tl_let.lower(ctx)?.into(),
        };
        Ok(decl)
    }
}

// Self Parameter
//
//

fn lower_self_param<T, F: FnOnce(&mut Ctx, ast::SelfParam) -> Result<T, LoweringError>>(
    self_param: &cst::decls::SelfParam,
    ctx: &mut Ctx,
    f: F,
) -> Result<T, LoweringError> {
    let cst::decls::SelfParam { span, name, typ } = self_param;
    let typ_out = typ.lower(ctx)?;
    let typ_ctor =
        typ_out.to_typctor().ok_or(LoweringError::ExpectedTypCtor { span: span.to_miette() })?;
    ctx.bind_single(
        name.clone().unwrap_or_else(|| parser::cst::ident::Ident {
            span: Default::default(),
            id: "".to_owned(),
        }),
        |ctx| {
            f(
                ctx,
                ast::SelfParam {
                    info: Some(*span),
                    name: name.clone().map(|name| ast::VarBind { span: None, id: name.id }),
                    typ: typ_ctor,
                },
            )
        },
    )
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
fn lower_telescope<T, F>(
    tele: &cst::decls::Telescope,
    ctx: &mut Ctx,
    f: F,
) -> Result<T, LoweringError>
where
    F: FnOnce(&mut Ctx, ast::Telescope) -> Result<T, LoweringError>,
{
    let tel = desugar_telescope(tele);
    ctx.bind_fold(
        tel.0.iter(),
        Ok(vec![]),
        |ctx, params_out, param| {
            let mut params_out = params_out?;
            let cst::decls::Param { implicit, name, names: _, typ } = param; // The `names` field has been removed by `desugar_telescope`.
            let typ_out = typ.lower(ctx)?;
            let name = match name {
                BindingSite::Var { name, .. } => name.clone(),
                BindingSite::Wildcard { span } => {
                    parser::cst::ident::Ident { span: *span, id: "_".to_owned() }
                }
            };
            let name = VarBind { span: Some(name.span), id: name.id.clone() };
            let param_out = ast::Param { implicit: *implicit, name, typ: typ_out };
            params_out.push(param_out);
            Ok(params_out)
        },
        |ctx, params| f(ctx, params.map(|params| ast::Telescope { params })?),
    )
}
