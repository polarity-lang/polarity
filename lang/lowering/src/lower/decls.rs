use std::rc::Rc;

use miette_util::ToMiette;
use parser::cst;
use parser::cst::exp::BindingSite;
use syntax::ast;
use syntax::ast::lookup_table::DeclKind;
use syntax::ast::lookup_table::DeclMeta;

use super::*;

impl Lower for cst::decls::DocComment {
    type Target = ast::DocComment;

    fn lower(&self, _ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        Ok(ast::DocComment { docs: self.docs.clone() })
    }
}

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

impl Lower for cst::decls::Decl {
    type Target = ();

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let decl = match self {
            cst::decls::Decl::Data(data) => ast::Decl::Data(data.lower(ctx)?),
            cst::decls::Decl::Codata(codata) => ast::Decl::Codata(codata.lower(ctx)?),
            cst::decls::Decl::Def(def) => ast::Decl::Def(def.lower(ctx)?),
            cst::decls::Decl::Codef(codef) => ast::Decl::Codef(codef.lower(ctx)?),
            cst::decls::Decl::Let(tl_let) => ast::Decl::Let(tl_let.lower(ctx)?),
        };
        ctx.add_decl(decl)?;
        Ok(())
    }
}

impl Lower for cst::decls::Data {
    type Target = ast::Data;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::decls::Data { span, doc, name, attr, params, ctors } = self;

        let ctor_decls = ctors.lower(ctx)?.into_iter().map(ast::Decl::Ctor);

        let ctor_names = ctors.iter().map(|ctor| ctor.name.id.clone()).collect();

        ctx.add_decls(ctor_decls)?;

        Ok(ast::Data {
            span: Some(*span),
            doc: doc.lower(ctx)?,
            name: name.id.clone(),
            attr: attr.lower(ctx)?,
            typ: Rc::new(lower_telescope(params, ctx, |_, out| Ok(out))?),
            ctors: ctor_names,
        })
    }
}

impl Lower for cst::decls::Codata {
    type Target = ast::Codata;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::decls::Codata { span, doc, name, attr, params, dtors } = self;

        let dtor_decls = dtors.lower(ctx)?.into_iter().map(ast::Decl::Dtor);

        let dtor_names = dtors.iter().map(|dtor| dtor.name.id.clone()).collect();

        ctx.add_decls(dtor_decls)?;

        Ok(ast::Codata {
            span: Some(*span),
            doc: doc.lower(ctx)?,
            name: name.id.clone(),
            attr: attr.lower(ctx)?,
            typ: Rc::new(lower_telescope(params, ctx, |_, out| Ok(out))?),
            dtors: dtor_names,
        })
    }
}

impl Lower for cst::decls::Ctor {
    type Target = ast::Ctor;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::decls::Ctor { span, doc, name, params, typ } = self;

        let typ_name = match ctx.lookup_top_level_decl(name, span)? {
            DeclMeta::Ctor { ret_typ } => ret_typ,
            other => {
                return Err(LoweringError::InvalidDeclarationKind {
                    name: name.id.clone(),
                    expected: DeclKind::Ctor,
                    actual: other.kind(),
                })
            }
        };

        let type_arity = match ctx
            .lookup_top_level_decl(&parser::cst::ident::Ident { id: typ_name.clone() }, span)?
        {
            DeclMeta::Data { arity } => arity,
            other => {
                return Err(LoweringError::InvalidDeclarationKind {
                    name: name.id.clone(),
                    expected: DeclKind::Data,
                    actual: other.kind(),
                })
            }
        };

        lower_telescope(params, ctx, |ctx, params| {
            // If the type constructor does not take any arguments, it can be left out
            let typ = match typ {
                Some(typ) => typ
                    .lower(ctx)?
                    .to_typctor()
                    .ok_or(LoweringError::ExpectedTypCtor { span: span.to_miette() })?,
                None => {
                    if type_arity == 0 {
                        ast::TypCtor {
                            span: None,
                            name: typ_name.clone(),
                            args: ast::Args { args: vec![] },
                        }
                    } else {
                        return Err(LoweringError::MustProvideArgs {
                            xtor: name.clone(),
                            typ: parser::cst::ident::Ident { id: typ_name.clone() },
                            span: span.to_miette(),
                        });
                    }
                }
            };

            Ok(ast::Ctor {
                span: Some(*span),
                doc: doc.lower(ctx)?,
                name: name.id.clone(),
                params,
                typ,
            })
        })
    }
}

impl Lower for cst::decls::Dtor {
    type Target = ast::Dtor;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::decls::Dtor { span, doc, name, params, destructee, ret_typ } = self;

        let typ_name = match ctx.lookup_top_level_decl(name, span)? {
            DeclMeta::Dtor { self_typ } => self_typ,
            other => {
                return Err(LoweringError::InvalidDeclarationKind {
                    name: name.id.clone(),
                    expected: DeclKind::Dtor,
                    actual: other.kind(),
                })
            }
        };

        let type_arity = match ctx
            .lookup_top_level_decl(&parser::cst::ident::Ident { id: typ_name.clone() }, span)?
        {
            DeclMeta::Codata { arity } => arity,
            other => {
                return Err(LoweringError::InvalidDeclarationKind {
                    name: name.id.clone(),
                    expected: DeclKind::Codata,
                    actual: other.kind(),
                })
            }
        };

        lower_telescope(params, ctx, |ctx, params| {
            // If the type constructor does not take any arguments, it can be left out
            let on_typ = match &destructee.typ {
                Some(on_typ) => on_typ.clone(),
                None => {
                    if type_arity == 0 {
                        cst::exp::Call {
                            span: Default::default(),
                            name: parser::cst::ident::Ident { id: typ_name.clone() },
                            args: vec![],
                        }
                    } else {
                        return Err(LoweringError::MustProvideArgs {
                            xtor: name.clone(),
                            typ: parser::cst::ident::Ident { id: typ_name.clone() },
                            span: span.to_miette(),
                        });
                    }
                }
            };

            let self_param = cst::decls::SelfParam {
                span: destructee.span,
                name: destructee.name.clone(),
                typ: on_typ,
            };

            lower_self_param(&self_param, ctx, |ctx, self_param| {
                Ok(ast::Dtor {
                    span: Some(*span),
                    doc: doc.lower(ctx)?,
                    name: name.id.clone(),
                    params,
                    self_param,
                    ret_typ: ret_typ.lower(ctx)?,
                })
            })
        })
    }
}

impl Lower for cst::decls::Def {
    type Target = ast::Def;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::decls::Def { span, doc, name, attr, params, scrutinee, ret_typ, cases } = self;

        let self_param: cst::decls::SelfParam = scrutinee.clone().into();

        lower_telescope(params, ctx, |ctx, params| {
            let cases = cases.lower(ctx)?;
            lower_self_param(&self_param, ctx, |ctx, self_param| {
                Ok(ast::Def {
                    span: Some(*span),
                    doc: doc.lower(ctx)?,
                    name: name.id.clone(),
                    attr: attr.lower(ctx)?,
                    params,
                    self_param,
                    ret_typ: ret_typ.lower(ctx)?,
                    cases,
                })
            })
        })
    }
}

impl Lower for cst::decls::Codef {
    type Target = ast::Codef;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::decls::Codef { span, doc, name, attr, params, typ, cases, .. } = self;

        lower_telescope(params, ctx, |ctx, params| {
            let typ = typ.lower(ctx)?;
            let typ_ctor = typ
                .to_typctor()
                .ok_or(LoweringError::ExpectedTypCtor { span: span.to_miette() })?;
            Ok(ast::Codef {
                span: Some(*span),
                doc: doc.lower(ctx)?,
                name: name.id.clone(),
                attr: attr.lower(ctx)?,
                params,
                typ: typ_ctor,
                cases: cases.lower(ctx)?,
            })
        })
    }
}

impl Lower for cst::decls::Let {
    type Target = ast::Let;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::decls::Let { span, doc, name, attr, params, typ, body } = self;

        lower_telescope(params, ctx, |ctx, params| {
            Ok(ast::Let {
                span: Some(*span),
                doc: doc.lower(ctx)?,
                name: name.id.clone(),
                attr: attr.lower(ctx)?,
                params,
                typ: typ.lower(ctx)?,
                body: body.lower(ctx)?,
            })
        })
    }
}

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
        name.clone().unwrap_or_else(|| parser::cst::ident::Ident { id: "".to_owned() }),
        |ctx| {
            f(
                ctx,
                ast::SelfParam {
                    info: Some(*span),
                    name: name.clone().map(|name| name.id),
                    typ: typ_ctor,
                },
            )
        },
    )
}

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
                BindingSite::Wildcard { .. } => parser::cst::ident::Ident { id: "_".to_owned() },
            };
            let param_out = ast::Param { implicit: *implicit, name: name.id, typ: typ_out };
            params_out.push(param_out);
            Ok(params_out)
        },
        |ctx, params| f(ctx, params.map(|params| ast::Telescope { params })?),
    )
}
