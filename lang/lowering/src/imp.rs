use std::rc::Rc;

use codespan::Span;
use num_bigint::BigUint;

use miette_util::ToMiette;
use parser::cst;
use parser::cst::exp::BindingSite;
use parser::cst::exp::Ident;
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

pub fn lower_prg(prg: &cst::decls::Prg) -> Result<ust::Prg, LoweringError> {
    let cst::decls::Prg { items } = prg;

    let (top_level_map, lookup_table) = build_lookup_table(items)?;

    let mut ctx = Ctx::empty(top_level_map);
    // Lower definitions
    for item in items {
        item.lower(&mut ctx)?;
    }

    Ok(ust::Prg { decls: ust::Decls { map: ctx.decls_map, lookup_table } })
}

/// Build the structure tracking the declaration order in the source code
fn build_lookup_table(
    items: &[cst::decls::Decl],
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
            cst::decls::Decl::Data(data) => {
                // top_level_map
                add_top_level_decl(
                    &data.name,
                    &data.span,
                    DeclMeta::Data { arity: data.params.len() },
                )?;
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
            cst::decls::Decl::Codata(codata) => {
                // top_level_map
                add_top_level_decl(
                    &codata.name,
                    &codata.span,
                    DeclMeta::Codata { arity: codata.params.len() },
                )?;
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
            cst::decls::Decl::Def(def) => {
                // top_level_map
                add_top_level_decl(&def.name, &def.span, DeclMeta::Def)?;

                // lookup_table
                let type_name = def.scrutinee.typ.name.clone();
                lookup_table.add_def(type_name, def.name.to_owned());
            }
            cst::decls::Decl::Codef(codef) => {
                // top_level_map
                add_top_level_decl(&codef.name, &codef.span, DeclMeta::Codef)?;

                // lookup_table
                let type_name = codef.typ.name.clone();
                lookup_table.add_def(type_name, codef.name.to_owned())
            }
            cst::decls::Decl::Let(tl_let) => {
                // top_level_map
                add_top_level_decl(&tl_let.name, &tl_let.span, DeclMeta::Let)?;

                lookup_table.add_let(tl_let.name.clone());
            }
        }
    }

    Ok((top_level_map, lookup_table))
}

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
            info: Some(*span),
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
            info: Some(*span),
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

            Ok(ust::Ctor {
                info: Some(*span),
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
                    info: Some(*span),
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
                    info: Some(*span),
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
                info: Some(*span),
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
                info: Some(*span),
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

impl Lower for cst::exp::Match {
    type Target = ust::Match;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Match { span, cases, omit_absurd } = self;

        Ok(ust::Match { info: Some(*span), cases: cases.lower(ctx)?, omit_absurd: *omit_absurd })
    }
}

impl Lower for cst::exp::Case {
    type Target = ust::Case;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Case { span, name, args, body } = self;

        lower_telescope_inst(args, ctx, |ctx, args| {
            Ok(ust::Case { info: Some(*span), name: name.clone(), args, body: body.lower(ctx)? })
        })
    }
}

impl Lower for cst::decls::TypApp {
    type Target = ust::TypApp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::decls::TypApp { span, name, args } = self;

        Ok(ust::TypApp {
            info: Some(*span),
            name: name.clone(),
            args: ust::Args { args: args.lower(ctx)? },
        })
    }
}

impl Lower for cst::exp::Call {
    type Target = ust::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Call { span, name, args } = self;
        match ctx.lookup(name, span)? {
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
                DeclKind::Let => Err(LoweringError::Impossible {
                    message: "Referencing top-level let definitions is not implemented, yet"
                        .to_owned(),
                    span: Some(span.to_miette()),
                }),
            },
        }
    }
}

impl Lower for cst::exp::DotCall {
    type Target = ust::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::DotCall { span, exp, name, args } = self;

        match ctx.lookup(name, span)? {
            Elem::Bound(_) => {
                Err(LoweringError::CannotUseAsDtor { name: name.clone(), span: span.to_miette() })
            }
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
        }
    }
}

impl Lower for cst::exp::Anno {
    type Target = ust::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Anno { span, exp, typ } = self;
        Ok(ust::Exp::Anno { info: Some(*span), exp: exp.lower(ctx)?, typ: typ.lower(ctx)? })
    }
}

impl Lower for cst::exp::Type {
    type Target = ust::Exp;

    fn lower(&self, _ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Type { span } = self;
        Ok(ust::Exp::Type { info: Some(*span) })
    }
}

impl Lower for cst::exp::LocalMatch {
    type Target = ust::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::LocalMatch { span, name, on_exp, motive, body } = self;
        Ok(ust::Exp::Match {
            info: Some(*span),
            ctx: (),
            name: ctx.unique_label(name.to_owned(), span)?,
            on_exp: on_exp.lower(ctx)?,
            motive: motive.lower(ctx)?,
            ret_typ: (),
            body: body.lower(ctx)?,
        })
    }
}

impl Lower for cst::exp::LocalComatch {
    type Target = ust::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::LocalComatch { span, name, is_lambda_sugar, body } = self;
        Ok(ust::Exp::Comatch {
            info: Some(*span),
            ctx: (),
            name: ctx.unique_label(name.to_owned(), span)?,
            is_lambda_sugar: *is_lambda_sugar,
            body: body.lower(ctx)?,
        })
    }
}

impl Lower for cst::exp::Hole {
    type Target = ust::Exp;

    fn lower(&self, _ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Hole { span } = self;
        Ok(ust::Exp::Hole { info: Some(*span) })
    }
}

impl Lower for cst::exp::NatLit {
    type Target = ust::Exp;

    fn lower(&self, _ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::NatLit { span, val } = self;
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
}

impl Lower for cst::exp::Fun {
    type Target = ust::Exp;
    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Fun { span, from, to } = self;
        Ok(ust::Exp::TypCtor {
            info: Some(*span),
            name: "Fun".to_owned(),
            args: ust::Args { args: vec![from.lower(ctx)?, to.lower(ctx)?] },
        })
    }
}

impl Lower for cst::exp::Lam {
    type Target = ust::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Lam { span, var, body } = self;
        let comatch = cst::exp::Exp::LocalComatch(cst::exp::LocalComatch {
            span: *span,
            name: None,
            is_lambda_sugar: true,
            body: cst::exp::Match {
                span: *span,
                cases: vec![cst::exp::Case {
                    span: *span,
                    name: "ap".to_owned(),
                    args: cst::exp::TelescopeInst(vec![
                        cst::exp::ParamInst {
                            span: Default::default(),
                            name: BindingSite::Wildcard,
                        },
                        cst::exp::ParamInst {
                            span: Default::default(),
                            name: BindingSite::Wildcard,
                        },
                        var.clone(),
                    ]),
                    body: Some(body.clone()),
                }],
                omit_absurd: false,
            },
        });
        comatch.lower(ctx)
    }
}

impl Lower for cst::exp::Exp {
    type Target = ust::Exp;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        match self {
            cst::exp::Exp::Call(e) => e.lower(ctx),
            cst::exp::Exp::DotCall(e) => e.lower(ctx),
            cst::exp::Exp::Anno(e) => e.lower(ctx),
            cst::exp::Exp::Type(e) => e.lower(ctx),
            cst::exp::Exp::LocalMatch(e) => e.lower(ctx),
            cst::exp::Exp::LocalComatch(e) => e.lower(ctx),
            cst::exp::Exp::Hole(e) => e.lower(ctx),
            cst::exp::Exp::NatLit(e) => e.lower(ctx),
            cst::exp::Exp::Fun(e) => e.lower(ctx),
            cst::exp::Exp::Lam(e) => e.lower(ctx),
        }
    }
}

fn bs_to_name(bs: &cst::exp::BindingSite) -> Ident {
    match bs {
        BindingSite::Var { name } => name.clone(),
        BindingSite::Wildcard => "_".to_owned(),
    }
}
impl Lower for cst::exp::Motive {
    type Target = ust::Motive;

    fn lower(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::exp::Motive { span, param, ret_typ } = self;

        Ok(ust::Motive {
            info: Some(*span),
            param: ust::ParamInst {
                info: Some(param.span),
                name: bs_to_name(&param.name),
                typ: (),
            },
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
    tel_inst: &cst::exp::TelescopeInst,
    ctx: &mut Ctx,
    f: F,
) -> Result<T, LoweringError> {
    ctx.bind_fold(
        tel_inst.0.iter(),
        Ok(vec![]),
        |_ctx, params_out, param| {
            let mut params_out = params_out?;
            let cst::exp::ParamInst { span, name } = param;
            let param_out =
                ust::ParamInst { info: Some(*span), name: bs_to_name(name).clone(), typ: () };
            params_out.push(param_out);
            Ok(params_out)
        },
        |ctx, params| f(ctx, params.map(|params| ust::TelescopeInst { params })?),
    )
}
