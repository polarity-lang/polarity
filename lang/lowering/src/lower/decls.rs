use std::rc::Rc;

use miette_util::ToMiette;
use parser::cst;
use parser::cst::exp::BindingSite;
use syntax::ctx::BindContext;
use syntax::generic::lookup_table::DeclKind;
use syntax::generic::lookup_table::DeclMeta;
use syntax::ust;

use super::*;

impl Lower for cst::decls::DocComment {
    type Target = ust::DocComment;

    fn lower(&self, _ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        Ok(ust::DocComment { docs: self.docs.clone() })
    }
}

impl Lower for cst::decls::Attribute {
    type Target = ust::Attribute;

    fn lower(&self, _ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        Ok(ust::Attribute { attrs: self.attrs.clone() })
    }
}

impl Lower for cst::decls::Decl {
    type Target = ();

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let decl = match self {
            cst::decls::Decl::Data(data) => ust::Decl::Data(data.lower(ctx)?),
            cst::decls::Decl::Codata(codata) => ust::Decl::Codata(codata.lower(ctx)?),
            cst::decls::Decl::Def(def) => ust::Decl::Def(def.lower(ctx)?),
            cst::decls::Decl::Codef(codef) => ust::Decl::Codef(codef.lower(ctx)?),
            cst::decls::Decl::Let(tl_let) => ust::Decl::Let(tl_let.lower(ctx)?),
        };
        ctx.add_decl(decl)?;
        Ok(())
    }
}

impl Lower for cst::decls::Data {
    type Target = ust::Data;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::decls::Data { span, doc, name, attr, params, ctors } = self;

        let ctor_decls = ctors.lower(ctx)?.into_iter().map(ust::Decl::Ctor);

        let ctor_names = ctors.iter().map(|ctor| ctor.name.clone()).collect();

        ctx.add_decls(ctor_decls)?;

        Ok(ust::Data {
            span: Some(*span),
            doc: doc.lower(ctx)?,
            name: name.clone(),
            attr: attr.lower(ctx)?,
            typ: Rc::new(ust::TypAbs { params: lower_telescope(params, ctx, |_, out| Ok(out))? }),
            ctors: ctor_names,
        })
    }
}

impl Lower for cst::decls::Codata {
    type Target = ust::Codata;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::decls::Codata { span, doc, name, attr, params, dtors } = self;

        let dtor_decls = dtors.lower(ctx)?.into_iter().map(ust::Decl::Dtor);

        let dtor_names = dtors.iter().map(|dtor| dtor.name.clone()).collect();

        ctx.add_decls(dtor_decls)?;

        Ok(ust::Codata {
            span: Some(*span),
            doc: doc.lower(ctx)?,
            name: name.clone(),
            attr: attr.lower(ctx)?,
            typ: Rc::new(ust::TypAbs { params: lower_telescope(params, ctx, |_, out| Ok(out))? }),
            dtors: dtor_names,
        })
    }
}

impl Lower for cst::decls::Ctor {
    type Target = ust::Ctor;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::decls::Ctor { span, doc, name, params, typ } = self;

        let typ_name = match ctx.lookup_top_level_decl(name, span)? {
            DeclMeta::Ctor { ret_typ } => ret_typ,
            other => {
                return Err(LoweringError::InvalidDeclarationKind {
                    name: name.clone(),
                    expected: DeclKind::Ctor,
                    actual: other.kind(),
                })
            }
        };

        let type_arity = match ctx.lookup_top_level_decl(&typ_name, span)? {
            DeclMeta::Data { arity } => arity,
            other => {
                return Err(LoweringError::InvalidDeclarationKind {
                    name: name.clone(),
                    expected: DeclKind::Data,
                    actual: other.kind(),
                })
            }
        };

        lower_telescope(params, ctx, |ctx, params| {
            // If the type constructor does not take any arguments, it can be left out
            let typ = match typ {
                Some(typ) => typ.lower(ctx)?,
                None => {
                    if type_arity == 0 {
                        ust::TypCtor {
                            span: None,
                            name: typ_name.clone(),
                            args: ust::Args { args: vec![] },
                        }
                    } else {
                        return Err(LoweringError::MustProvideArgs {
                            xtor: name.clone(),
                            typ: typ_name.clone(),
                            span: span.to_miette(),
                        });
                    }
                }
            };

            Ok(ust::Ctor {
                span: Some(*span),
                doc: doc.lower(ctx)?,
                name: name.clone(),
                params,
                typ,
            })
        })
    }
}

impl Lower for cst::decls::Dtor {
    type Target = ust::Dtor;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::decls::Dtor { span, doc, name, params, destructee, ret_typ } = self;

        let typ_name = match ctx.lookup_top_level_decl(name, span)? {
            DeclMeta::Dtor { self_typ } => self_typ,
            other => {
                return Err(LoweringError::InvalidDeclarationKind {
                    name: name.clone(),
                    expected: DeclKind::Dtor,
                    actual: other.kind(),
                })
            }
        };

        let type_arity = match ctx.lookup_top_level_decl(&typ_name, span)? {
            DeclMeta::Codata { arity } => arity,
            other => {
                return Err(LoweringError::InvalidDeclarationKind {
                    name: name.clone(),
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
                        cst::decls::TypApp {
                            span: Default::default(),
                            name: typ_name.clone(),
                            args: vec![],
                        }
                    } else {
                        return Err(LoweringError::MustProvideArgs {
                            xtor: name.clone(),
                            typ: typ_name.clone(),
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
                Ok(ust::Dtor {
                    span: Some(*span),
                    doc: doc.lower(ctx)?,
                    name: name.clone(),
                    params,
                    self_param,
                    ret_typ: ret_typ.lower(ctx)?,
                })
            })
        })
    }
}

impl Lower for cst::decls::Def {
    type Target = ust::Def;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::decls::Def { span, doc, name, attr, params, scrutinee, ret_typ, body } = self;

        let self_param: cst::decls::SelfParam = scrutinee.clone().into();

        lower_telescope(params, ctx, |ctx, params| {
            let body = body.lower(ctx)?;

            lower_self_param(&self_param, ctx, |ctx, self_param| {
                Ok(ust::Def {
                    span: Some(*span),
                    doc: doc.lower(ctx)?,
                    name: name.clone(),
                    attr: attr.lower(ctx)?,
                    params,
                    self_param,
                    ret_typ: ret_typ.lower(ctx)?,
                    body,
                })
            })
        })
    }
}

impl Lower for cst::decls::Codef {
    type Target = ust::Codef;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::decls::Codef { span, doc, name, attr, params, typ, body, .. } = self;

        lower_telescope(params, ctx, |ctx, params| {
            Ok(ust::Codef {
                span: Some(*span),
                doc: doc.lower(ctx)?,
                name: name.clone(),
                attr: attr.lower(ctx)?,
                params,
                typ: typ.lower(ctx)?,
                body: body.lower(ctx)?,
            })
        })
    }
}

impl Lower for cst::decls::Let {
    type Target = ust::Let;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::decls::Let { span, doc, name, attr, params, typ, body } = self;

        lower_telescope(params, ctx, |ctx, params| {
            Ok(ust::Let {
                span: Some(*span),
                doc: doc.lower(ctx)?,
                name: name.clone(),
                attr: attr.lower(ctx)?,
                params,
                typ: typ.lower(ctx)?,
                body: body.lower(ctx)?,
            })
        })
    }
}

impl Lower for cst::decls::TypApp {
    type Target = ust::TypCtor;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::decls::TypApp { span, name, args } = self;

        Ok(ust::TypCtor {
            span: Some(*span),
            name: name.clone(),
            args: ust::Args { args: args.lower(ctx)? },
        })
    }
}

fn lower_self_param<T, F: FnOnce(&mut Ctx, ust::SelfParam) -> Result<T, LoweringError>>(
    self_param: &cst::decls::SelfParam,
    ctx: &mut Ctx,
    f: F,
) -> Result<T, LoweringError> {
    let cst::decls::SelfParam { span, name, typ } = self_param;
    let typ_out = typ.lower(ctx)?;
    ctx.bind_single(name.clone().unwrap_or_default(), |ctx| {
        f(ctx, ust::SelfParam { info: Some(*span), name: name.clone(), typ: typ_out })
    })
}

fn desugar_telescope(tel: &cst::decls::Telescope) -> cst::decls::Telescope {
    let cst::decls::Telescope(params) = tel;
    let params: Vec<cst::decls::Param> = params.iter().flat_map(desugar_param).collect();
    cst::decls::Telescope(params)
}
fn desugar_param(param: &cst::decls::Param) -> Vec<cst::decls::Param> {
    let cst::decls::Param { name, names, typ } = param;
    let mut params: Vec<cst::decls::Param> =
        vec![cst::decls::Param { name: name.clone(), names: vec![], typ: typ.clone() }];
    for extra_name in names {
        params.push(cst::decls::Param {
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
    F: FnOnce(&mut Ctx, ust::Telescope) -> Result<T, LoweringError>,
{
    let tel = desugar_telescope(tele);
    ctx.bind_fold(
        tel.0.iter(),
        Ok(vec![]),
        |ctx, params_out, param| {
            let mut params_out = params_out?;
            let cst::decls::Param { name, names: _, typ } = param; // The `names` field has been removed by `desugar_telescope`.
            let typ_out = typ.lower(ctx)?;
            let name = match name {
                BindingSite::Var { name, .. } => name.clone(),
                BindingSite::Wildcard { .. } => "_".to_owned(),
            };
            let param_out = ust::Param { name, typ: typ_out };
            params_out.push(param_out);
            Ok(params_out)
        },
        |ctx, params| f(ctx, params.map(|params| ust::Telescope { params })?),
    )
}
