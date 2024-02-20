use std::rc::Rc;

use codespan::Span;
use num_bigint::BigUint;

use data::HashMap;
use miette_util::ToMiette;
use parser::cst;
use parser::cst::BindingSite;
use parser::cst::Ident;
use syntax::common::*;
use syntax::ctx::BindContext;
use syntax::generic::lookup_table;
use syntax::generic::lookup_table::DeclKind;
use syntax::generic::lookup_table::DeclMeta;
use syntax::ust;

use super::ctx::*;
use super::result::*;

pub trait Lower {
    type Target;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError>;
}

pub fn lower_prg(prg: &cst::Prg) -> Result<ust::Prg, LoweringError> {
    let cst::Prg { items, exp } = prg;

    let (top_level_map, lookup_table) = build_lookup_table(items)?;

    let mut ctx = Ctx::empty(top_level_map);
    // Lower definitions
    for item in items {
        item.lower(&mut ctx)?;
    }

    let exp = exp.lower(&mut ctx)?;

    Ok(ust::Prg { decls: ust::Decls { map: ctx.decls_map, lookup_table }, exp })
}

/// Build the structure tracking the declaration order in the source code
fn build_lookup_table(
    items: &[cst::Decl],
) -> Result<(HashMap<Ident, DeclMeta>, lookup_table::LookupTable), LoweringError> {
    let mut lookup_table = lookup_table::LookupTable::default();
    let mut top_level_map = HashMap::default();

    let mut add_top_level_decl = |name: &Ident, span: &Span, decl_kind: DeclMeta| {
        if top_level_map.contains_key(name) {
            return Err(LoweringError::AlreadyDefined {
                name: name.to_owned(),
                span: Some(span.to_miette()),
            });
        }
        top_level_map.insert(name.clone(), decl_kind);
        Ok(())
    };

    for item in items {
        match item {
            cst::Decl::Data(data) => {
                // top_level_map
                add_top_level_decl(&data.name, &data.span, DeclMeta::from(data))?;
                for ctor in &data.ctors {
                    add_top_level_decl(
                        &ctor.name,
                        &ctor.span,
                        DeclMeta::Ctor { ret_typ: data.name.clone() },
                    )?;
                }

                // lookup_table
                let mut typ_decl = lookup_table.add_type_decl(data.name.clone());
                let xtors = data.ctors.iter().map(|ctor| ctor.name.clone());
                typ_decl.set_xtors(xtors);
            }
            cst::Decl::Codata(codata) => {
                // top_level_map
                add_top_level_decl(&codata.name, &codata.span, DeclMeta::from(codata))?;
                for dtor in &codata.dtors {
                    add_top_level_decl(
                        &dtor.name,
                        &dtor.span,
                        DeclMeta::Dtor { self_typ: codata.name.clone() },
                    )?;
                }

                // lookup_table
                let mut typ_decl = lookup_table.add_type_decl(codata.name.clone());
                let xtors = codata.dtors.iter().map(|ctor| ctor.name.clone());
                typ_decl.set_xtors(xtors);
            }
            cst::Decl::Def(def) => {
                // top_level_map
                add_top_level_decl(&def.name, &def.span, DeclMeta::from(def))?;

                // lookup_table
                let type_name = def.scrutinee.typ.name.clone();
                lookup_table.add_def(type_name, def.name.to_owned());
            }
            cst::Decl::Codef(codef) => {
                // top_level_map
                add_top_level_decl(&codef.name, &codef.span, DeclMeta::from(codef))?;

                // lookup_table
                let type_name = codef.typ.name.clone();
                lookup_table.add_def(type_name, codef.name.to_owned())
            }
        }
    }

    Ok((top_level_map, lookup_table))
}

impl Lower for cst::Decl {
    type Target = ();

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let decl = match self {
            cst::Decl::Data(data) => ust::Decl::Data(data.lower(ctx)?),
            cst::Decl::Codata(codata) => ust::Decl::Codata(codata.lower(ctx)?),
            cst::Decl::Def(def) => ust::Decl::Def(def.lower(ctx)?),
            cst::Decl::Codef(codef) => ust::Decl::Codef(codef.lower(ctx)?),
        };
        ctx.add_decl(decl)?;
        Ok(())
    }
}

impl Lower for cst::Data {
    type Target = ust::Data;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::Data { span, doc, name, hidden, params, ctors } = self;

        let ctor_decls = ctors.lower(ctx)?.into_iter().map(ust::Decl::Ctor);

        let ctor_names = ctors.iter().map(|ctor| ctor.name.clone()).collect();

        ctx.add_decls(ctor_decls)?;

        Ok(ust::Data {
            info: Some(*span),
            doc: doc.clone(),
            name: name.clone(),
            hidden: *hidden,
            typ: Rc::new(ust::TypAbs { params: lower_telescope(params, ctx, |_, out| Ok(out))? }),
            ctors: ctor_names,
        })
    }
}

impl Lower for cst::Codata {
    type Target = ust::Codata;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::Codata { span, doc, name, hidden, params, dtors } = self;

        let dtor_decls = dtors.lower(ctx)?.into_iter().map(ust::Decl::Dtor);

        let dtor_names = dtors.iter().map(|dtor| dtor.name.clone()).collect();

        ctx.add_decls(dtor_decls)?;

        Ok(ust::Codata {
            info: Some(*span),
            doc: doc.clone(),
            name: name.clone(),
            hidden: *hidden,
            typ: Rc::new(ust::TypAbs { params: lower_telescope(params, ctx, |_, out| Ok(out))? }),
            dtors: dtor_names,
        })
    }
}

impl Lower for cst::Ctor {
    type Target = ust::Ctor;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::Ctor { span, doc, name, params, typ } = self;

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
                        ust::TypApp {
                            info: None,
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

            Ok(ust::Ctor { info: Some(*span), doc: doc.clone(), name: name.clone(), params, typ })
        })
    }
}

impl Lower for cst::Dtor {
    type Target = ust::Dtor;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::Dtor { span, doc, name, params, destructee, ret_typ } = self;

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
                        cst::TypApp {
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

            let self_param = cst::SelfParam {
                span: destructee.span,
                name: destructee.name.clone(),
                typ: on_typ,
            };

            lower_self_param(&self_param, ctx, |ctx, self_param| {
                Ok(ust::Dtor {
                    info: Some(*span),
                    doc: doc.clone(),
                    name: name.clone(),
                    params,
                    self_param,
                    ret_typ: ret_typ.lower(ctx)?,
                })
            })
        })
    }
}

impl Lower for cst::Def {
    type Target = ust::Def;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::Def { span, doc, name, hidden, params, scrutinee, ret_typ, body } = self;

        let self_param: cst::SelfParam = scrutinee.clone().into();

        lower_telescope(params, ctx, |ctx, params| {
            let body = body.lower(ctx)?;

            lower_self_param(&self_param, ctx, |ctx, self_param| {
                Ok(ust::Def {
                    info: Some(*span),
                    doc: doc.clone(),
                    name: name.clone(),
                    hidden: *hidden,
                    params,
                    self_param,
                    ret_typ: ret_typ.lower(ctx)?,
                    body,
                })
            })
        })
    }
}

impl Lower for cst::Codef {
    type Target = ust::Codef;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::Codef { span, doc, name, hidden, params, typ, body, .. } = self;

        lower_telescope(params, ctx, |ctx, params| {
            Ok(ust::Codef {
                info: Some(*span),
                doc: doc.clone(),
                name: name.clone(),
                hidden: *hidden,
                params,
                typ: typ.lower(ctx)?,
                body: body.lower(ctx)?,
            })
        })
    }
}

impl Lower for cst::Match {
    type Target = ust::Match;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::Match { span, cases, omit_absurd } = self;

        Ok(ust::Match { info: Some(*span), cases: cases.lower(ctx)?, omit_absurd: *omit_absurd })
    }
}

impl Lower for cst::Case {
    type Target = ust::Case;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::Case { span, name, args, body } = self;

        lower_telescope_inst(args, ctx, |ctx, args| {
            Ok(ust::Case { info: Some(*span), name: name.clone(), args, body: body.lower(ctx)? })
        })
    }
}

impl Lower for cst::TypApp {
    type Target = ust::TypApp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::TypApp { span, name, args } = self;

        Ok(ust::TypApp {
            info: Some(*span),
            name: name.clone(),
            args: ust::Args { args: args.lower(ctx)? },
        })
    }
}

impl Lower for cst::Exp {
    type Target = ust::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        match self {
            cst::Exp::Call { span, name, args } => match ctx.lookup(name, span)? {
                Elem::Bound(lvl) => Ok(ust::Exp::Var {
                    info: Some(*span),
                    name: name.clone(),
                    ctx: (),
                    idx: ctx.level_to_index(lvl),
                }),
                Elem::Decl(meta) => match meta.kind() {
                    DeclKind::Data | DeclKind::Codata => Ok(ust::Exp::TypCtor {
                        info: Some(*span),
                        name: name.to_owned(),
                        args: ust::Args { args: args.lower(ctx)? },
                    }),
                    DeclKind::Def | DeclKind::Dtor => Err(LoweringError::MustUseAsDtor {
                        name: name.to_owned(),
                        span: span.to_miette(),
                    }),
                    DeclKind::Codef | DeclKind::Ctor => Ok(ust::Exp::Ctor {
                        info: Some(*span),
                        name: name.to_owned(),
                        args: ust::Args { args: args.lower(ctx)? },
                    }),
                },
            },
            cst::Exp::DotCall { span, exp, name, args } => match ctx.lookup(name, span)? {
                Elem::Bound(_) => Err(LoweringError::CannotUseAsDtor {
                    name: name.clone(),
                    span: span.to_miette(),
                }),
                Elem::Decl(meta) => match meta.kind() {
                    DeclKind::Def | DeclKind::Dtor => Ok(ust::Exp::Dtor {
                        info: Some(*span),
                        exp: exp.lower(ctx)?,
                        name: name.clone(),
                        args: ust::Args { args: args.lower(ctx)? },
                    }),
                    _ => Err(LoweringError::CannotUseAsDtor {
                        name: name.clone(),
                        span: span.to_miette(),
                    }),
                },
            },
            cst::Exp::Anno { span, exp, typ } => {
                Ok(ust::Exp::Anno { info: Some(*span), exp: exp.lower(ctx)?, typ: typ.lower(ctx)? })
            }
            cst::Exp::Type { span } => Ok(ust::Exp::Type { info: Some(*span) }),
            cst::Exp::Match { span, name, on_exp, motive, body } => Ok(ust::Exp::Match {
                info: Some(*span),
                ctx: (),
                name: ctx.unique_label(name.to_owned(), span)?,
                on_exp: on_exp.lower(ctx)?,
                motive: motive.lower(ctx)?,
                ret_typ: (),
                body: body.lower(ctx)?,
            }),
            cst::Exp::Comatch { span, name, is_lambda_sugar, body } => Ok(ust::Exp::Comatch {
                info: Some(*span),
                ctx: (),
                name: ctx.unique_label(name.to_owned(), span)?,
                is_lambda_sugar: *is_lambda_sugar,
                body: body.lower(ctx)?,
            }),
            cst::Exp::Hole { span } => Ok(ust::Exp::Hole { info: Some(*span) }),
            cst::Exp::NatLit { span, val } => {
                let mut out = ust::Exp::Ctor {
                    info: Some(*span),
                    name: "Z".to_owned(),
                    args: ust::Args { args: vec![] },
                };

                let mut i = BigUint::from(0usize);

                while &i != val {
                    i += 1usize;
                    out = ust::Exp::Ctor {
                        info: Some(*span),
                        name: "S".to_owned(),
                        args: ust::Args { args: vec![Rc::new(out)] },
                    };
                }

                Ok(out)
            }
            cst::Exp::Fun { span, from, to } => Ok(ust::Exp::TypCtor {
                info: Some(*span),
                name: "Fun".to_owned(),
                args: ust::Args { args: vec![from.lower(ctx)?, to.lower(ctx)?] },
            }),
            cst::Exp::Lam { span, var, body } => {
                let comatch = cst::Exp::Comatch {
                    span: *span,
                    name: None,
                    is_lambda_sugar: true,
                    body: cst::Match {
                        span: *span,
                        cases: vec![cst::Case {
                            span: *span,
                            name: "ap".to_owned(),
                            args: cst::TelescopeInst(vec![
                                cst::ParamInst {
                                    span: Default::default(),
                                    name: BindingSite::Wildcard,
                                },
                                cst::ParamInst {
                                    span: Default::default(),
                                    name: BindingSite::Wildcard,
                                },
                                var.clone(),
                            ]),
                            body: Some(body.clone()),
                        }],
                        omit_absurd: false,
                    },
                };
                comatch.lower(ctx)
            }
        }
    }
}

impl Lower for cst::Motive {
    type Target = ust::Motive;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::Motive { span, param, ret_typ } = self;

        Ok(ust::Motive {
            info: Some(*span),
            param: ust::ParamInst { info: Some(param.span), name: param.name().clone(), typ: () },
            ret_typ: ctx.bind_single(param, |ctx| ret_typ.lower(ctx))?,
        })
    }
}

impl<T: Lower> Lower for Option<T> {
    type Target = Option<T::Target>;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        self.as_ref().map(|x| x.lower(ctx)).transpose()
    }
}

impl<T: Lower> Lower for Vec<T> {
    type Target = Vec<T::Target>;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        self.iter().map(|x| x.lower(ctx)).collect()
    }
}

impl<T: Lower> Lower for Rc<T> {
    type Target = Rc<T::Target>;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        Ok(Rc::new((**self).lower(ctx)?))
    }
}

fn lower_self_param<T, F: FnOnce(&mut Ctx, ust::SelfParam) -> Result<T, LoweringError>>(
    self_param: &cst::SelfParam,
    ctx: &mut Ctx,
    f: F,
) -> Result<T, LoweringError> {
    let cst::SelfParam { span, name, typ } = self_param;
    let typ_out = typ.lower(ctx)?;
    ctx.bind_single(name.clone().unwrap_or_default(), |ctx| {
        f(ctx, ust::SelfParam { info: Some(*span), name: name.clone(), typ: typ_out })
    })
}

fn desugar_telescope(tel: &cst::Telescope) -> cst::Telescope {
    let cst::Telescope(params) = tel;
    let params: Vec<cst::Param> = params.iter().flat_map(desugar_param).collect();
    cst::Telescope(params)
}
fn desugar_param(param: &cst::Param) -> Vec<cst::Param> {
    let cst::Param { name, names, typ } = param;
    let mut params: Vec<cst::Param> =
        vec![cst::Param { name: name.clone(), names: vec![], typ: typ.clone() }];
    for extra_name in names {
        params.push(cst::Param { name: extra_name.clone(), names: vec![], typ: typ.clone() });
    }
    params
}

/// Lower a telescope
///
/// Execute a function `f` under the context where all binders
/// of the telescope are in scope.
fn lower_telescope<T, F>(tele: &cst::Telescope, ctx: &mut Ctx, f: F) -> Result<T, LoweringError>
where
    F: FnOnce(&mut Ctx, ust::Telescope) -> Result<T, LoweringError>,
{
    let tel = desugar_telescope(tele);
    ctx.bind_fold(
        tel.0.iter(),
        Ok(vec![]),
        |ctx, params_out, param| {
            let mut params_out = params_out?;
            let cst::Param { name, names: _, typ } = param; // The `names` field has been removed by `desugar_telescope`.
            let typ_out = typ.lower(ctx)?;
            let name = match name {
                BindingSite::Var { name } => name.clone(),
                BindingSite::Wildcard => "_".to_owned(),
            };
            let param_out = ust::Param { name, typ: typ_out };
            params_out.push(param_out);
            Ok(params_out)
        },
        |ctx, params| f(ctx, params.map(|params| ust::Telescope { params })?),
    )
}

fn lower_telescope_inst<T, F: FnOnce(&mut Ctx, ust::TelescopeInst) -> Result<T, LoweringError>>(
    tel_inst: &cst::TelescopeInst,
    ctx: &mut Ctx,
    f: F,
) -> Result<T, LoweringError> {
    ctx.bind_fold(
        tel_inst.0.iter(),
        Ok(vec![]),
        |_ctx, params_out, param| {
            let mut params_out = params_out?;
            let cst::ParamInst { span, name } = param;
            let param_out =
                ust::ParamInst { info: Some(*span), name: name.name().clone(), typ: () };
            params_out.push(param_out);
            Ok(params_out)
        },
        |ctx, params| f(ctx, params.map(|params| ust::TelescopeInst { params })?),
    )
}
