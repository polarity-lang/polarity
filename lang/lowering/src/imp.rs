use std::rc::Rc;

use syntax::ast;
use syntax::cst;
use syntax::named::Named;

use super::ctx::*;
use super::result::*;
use super::types::*;

pub fn lower(prg: &cst::Prg) -> Result<ast::Prg, LoweringError> {
    let cst::Prg { items, exp } = prg;
    let mut ctx = Ctx::empty();

    // Register names and metadata
    let (types, defs) = register_names(&mut ctx, &items[..])?;

    // Lower deferred definitions
    for typ in types {
        typ.lower_in_ctx(&mut ctx)?;
    }
    for def in defs {
        def.lower_in_ctx(&mut ctx)?;
    }

    let exp = exp.lower_in_ctx(&mut ctx)?;

    Ok(ast::Prg { decls: ctx.into_decls(), exp })
}

/// Register names for all top-level declarations
/// Returns definitions whose lowering has been deferred
fn register_names<'a>(
    ctx: &mut Ctx,
    items: &'a [cst::Item],
) -> Result<(Vec<&'a cst::TypDecl>, Vec<&'a cst::DefDecl>), LoweringError> {
    let mut types = Vec::new();
    let mut defs = Vec::new();

    for item in items {
        match item {
            cst::Item::Type(type_decl) => {
                register_type_name(ctx, type_decl)?;
                types.push(type_decl);
            }
            cst::Item::Impl(impl_decl) => {
                register_impl_meta(ctx, impl_decl)?;
                defs.extend(impl_decl.decls.iter());
            }
        }
    }

    Ok((types, defs))
}

fn register_type_name(ctx: &mut Ctx, type_decl: &cst::TypDecl) -> Result<(), LoweringError> {
    // Register type name in the context
    // Don't lower any of the contents, yet
    ctx.add_name(type_decl.name(), DeclKind::from(type_decl))?;
    // Register names for all xtors
    match type_decl {
        cst::TypDecl::Data(data) => {
            for ctor in &data.ctors {
                ctx.add_name(&ctor.name, DeclKind::Ctor)?;
            }
        }
        cst::TypDecl::Codata(codata) => {
            for dtor in &codata.dtors {
                ctx.add_name(&dtor.name, DeclKind::Ctor)?;
            }
        }
    }
    Ok(())
}

fn register_impl_meta(ctx: &mut Ctx, impl_decl: &cst::Impl) -> Result<(), LoweringError> {
    // Add metadata of impl block to context
    // This does not lower any of the contents, yet
    impl_decl.lower_in_ctx(ctx)?;
    for def in &impl_decl.decls {
        // Add names for all contained definitions
        ctx.add_name(def.name(), DeclKind::from(def))?;
    }
    Ok(())
}

impl Lower for cst::TypDecl {
    type Target = ();

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let decl = match self {
            cst::TypDecl::Data(data) => ast::Decl::Data(data.lower_in_ctx(ctx)?),
            cst::TypDecl::Codata(codata) => ast::Decl::Codata(codata.lower_in_ctx(ctx)?),
        };
        ctx.add_decl(decl)?;
        Ok(())
    }
}

impl Lower for cst::Impl {
    type Target = ();

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::Impl { info, name, decls } = self;

        let impl_block = ast::Impl {
            info: info.lower_pure(),
            name: name.clone(),
            defs: decls.iter().map(Named::name).cloned().collect(),
        };

        ctx.add_impl_block(impl_block);

        Ok(())
    }
}

impl Lower for cst::DefDecl {
    type Target = ();

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let decl = match self {
            cst::DefDecl::Def(def) => ast::Decl::Def(def.lower_in_ctx(ctx)?),
            cst::DefDecl::Codef(codef) => ast::Decl::Codef(codef.lower_in_ctx(ctx)?),
        };
        ctx.add_decl(decl)?;
        Ok(())
    }
}

impl Lower for cst::Data {
    type Target = ast::Data;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::Data { info, name, params, ctors } = self;

        let ctor_decls = ctors.lower_in_ctx(ctx)?.into_iter().map(ast::Decl::Ctor);

        let ctor_names = ctors.iter().map(|ctor| ctor.name.clone()).collect();

        ctx.add_decls(ctor_decls)?;

        Ok(ast::Data {
            info: info.lower_pure(),
            name: name.clone(),
            typ: Rc::new(ast::TypAbs { params: params.lower_in_ctx(ctx)? }),
            ctors: ctor_names,
            impl_block: ctx.impl_block(name).cloned(),
        })
    }
}

impl Lower for cst::Codata {
    type Target = ast::Codata;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::Codata { info, name, params, dtors } = self;

        let dtor_decls = dtors.lower_in_ctx(ctx)?.into_iter().map(ast::Decl::Dtor);

        let dtor_names = dtors.iter().map(|dtor| dtor.name.clone()).collect();

        ctx.add_decls(dtor_decls)?;

        Ok(ast::Codata {
            info: info.lower_pure(),
            name: name.clone(),
            typ: Rc::new(ast::TypAbs { params: params.lower_in_ctx(ctx)? }),
            dtors: dtor_names,
            impl_block: ctx.impl_block(name).cloned(),
        })
    }
}

impl Lower for cst::Ctor {
    type Target = ast::Ctor;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::Ctor { info, name, params, typ } = self;

        params.lower_telescope(ctx, |ctx, params| {
            Ok(ast::Ctor {
                info: info.lower_pure(),
                name: name.clone(),
                params,
                typ: typ.lower_in_ctx(ctx)?,
            })
        })
    }
}

impl Lower for cst::Dtor {
    type Target = ast::Dtor;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::Dtor { info, name, params, on_typ, in_typ } = self;

        params.lower_telescope(ctx, |ctx, params| {
            Ok(ast::Dtor {
                info: info.lower_pure(),
                name: name.clone(),
                params,
                on_typ: on_typ.lower_in_ctx(ctx)?,
                in_typ: in_typ.lower_in_ctx(ctx)?,
            })
        })
    }
}

impl Lower for cst::Def {
    type Target = ast::Def;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::Def { info, name, params, on_typ, in_typ, body } = self;

        params.lower_telescope(ctx, |ctx, params| {
            Ok(ast::Def {
                info: info.lower_pure(),
                name: name.clone(),
                params,
                on_typ: on_typ.lower_in_ctx(ctx)?,
                in_typ: in_typ.lower_in_ctx(ctx)?,
                body: body.lower_in_ctx(ctx)?,
            })
        })
    }
}

impl Lower for cst::Codef {
    type Target = ast::Codef;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::Codef { info, name, params, typ, body } = self;

        params.lower_telescope(ctx, |ctx, params| {
            Ok(ast::Codef {
                info: info.lower_pure(),
                name: name.clone(),
                params,
                typ: typ.lower_in_ctx(ctx)?,
                body: body.lower_in_ctx(ctx)?,
            })
        })
    }
}

impl Lower for cst::Match {
    type Target = ast::Match;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::Match { info, cases } = self;

        Ok(ast::Match { info: info.lower_pure(), cases: cases.lower_in_ctx(ctx)? })
    }
}

impl Lower for cst::Comatch {
    type Target = ast::Comatch;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::Comatch { info, cases } = self;

        Ok(ast::Comatch { info: info.lower_pure(), cases: cases.lower_in_ctx(ctx)? })
    }
}

impl Lower for cst::Case {
    type Target = ast::Case;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::Case { info, name, args, body, eqns } = self;

        args.lower_telescope(ctx, |ctx, args| {
            eqns.lower_params(ctx, move |ctx, eqns| {
                Ok(ast::Case {
                    info: info.lower_pure(),
                    name: name.clone(),
                    args,
                    eqns,
                    body: body.lower_in_ctx(ctx)?,
                })
            })
        })
    }
}

impl Lower for cst::Cocase {
    type Target = ast::Cocase;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::Cocase { info, name, args, body, eqns } = self;

        args.lower_telescope(ctx, |ctx, args| {
            eqns.lower_params(ctx, |ctx, eqns| {
                Ok(ast::Cocase {
                    info: info.lower_pure(),
                    name: name.clone(),
                    args,
                    eqns,
                    body: body.lower_in_ctx(ctx)?,
                })
            })
        })
    }
}

impl Lower for cst::TypApp {
    type Target = ast::TypApp;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::TypApp { info, name, args: subst } = self;

        Ok(ast::TypApp {
            info: info.lower_pure(),
            name: name.clone(),
            args: subst.lower_in_ctx(ctx)?,
        })
    }
}

impl Lower for cst::Eqn {
    type Target = ast::Eqn;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::Eqn { info, lhs, rhs } = self;

        Ok(ast::Eqn {
            info: info.lower_pure(),
            lhs: lhs.lower_in_ctx(ctx)?,
            rhs: rhs.lower_in_ctx(ctx)?,
        })
    }
}

impl Lower for cst::Exp {
    type Target = ast::Exp;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        match self {
            cst::Exp::Call { info, name, args: subst } => match ctx.lookup(name)? {
                Elem::Bound(lvl) => Ok(ast::Exp::Var {
                    info: info.lower_pure(),
                    name: name.clone(),
                    idx: ctx.lower_bound(*lvl),
                }),
                Elem::Decl(decl_kind) => match decl_kind {
                    DeclKind::Codata | DeclKind::Data => Ok(ast::Exp::TypCtor {
                        info: info.lower_pure(),
                        name: name.to_owned(),
                        args: subst.lower_in_ctx(ctx)?,
                    }),
                    DeclKind::Def | DeclKind::Dtor => {
                        Err(LoweringError::MustUseAsDtor(name.to_owned()))
                    }
                    DeclKind::Codef | DeclKind::Ctor => Ok(ast::Exp::Ctor {
                        info: info.lower_pure(),
                        name: name.to_owned(),
                        args: subst.lower_in_ctx(ctx)?,
                    }),
                },
            },
            cst::Exp::DotCall { info, exp, name, args: subst } => Ok(ast::Exp::Dtor {
                info: info.lower_pure(),
                exp: exp.lower_in_ctx(ctx)?,
                name: name.clone(),
                args: subst.lower_in_ctx(ctx)?,
            }),
            cst::Exp::Anno { info, exp, typ } => Ok(ast::Exp::Anno {
                info: info.lower_pure(),
                exp: exp.lower_in_ctx(ctx)?,
                typ: typ.lower_in_ctx(ctx)?,
            }),
            cst::Exp::Type { info } => Ok(ast::Exp::Type { info: info.lower_pure() }),
        }
    }
}

impl<T: Lower> Lower for Option<T> {
    type Target = Option<T::Target>;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        self.as_ref().map(|x| x.lower_in_ctx(ctx)).transpose()
    }
}

impl<T: Lower> Lower for Vec<T> {
    type Target = Vec<T::Target>;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        self.iter().map(|x| x.lower_in_ctx(ctx)).collect()
    }
}

impl<T: Lower> Lower for Rc<T> {
    type Target = Rc<T::Target>;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        Ok(Rc::new((**self).lower_in_ctx(ctx)?))
    }
}

impl LowerParams for cst::EqnParams {
    type Target = ast::EqnParams;

    /// Lower a list of parameters
    ///
    /// Execute a function `f` under the context where all binders
    /// of the telescope are in scope.
    fn lower_params<T, F: FnOnce(&mut Ctx, Self::Target) -> Result<T, LoweringError>>(
        &self,
        ctx: &mut Ctx,
        f: F,
    ) -> Result<T, LoweringError> {
        ctx.bind_fold(
            self.iter(),
            Ok(ast::EqnParams::new()),
            |ctx, params_out, param| {
                let mut params_out = params_out?;
                let cst::EqnParam { name, eqn } = param;
                let eqn_out = eqn.lower_in_ctx(ctx)?;
                let param_out = ast::EqnParam { name: name.clone(), eqn: eqn_out };
                params_out.push(param_out);
                Ok(params_out)
            },
            |ctx, params| f(ctx, params?),
        )
    }
}

impl LowerTelescope for cst::Telescope {
    type Target = ast::Telescope;

    /// Lower a telescope
    ///
    /// Execute a function `f` under the context where all binders
    /// of the telescope are in scope.
    fn lower_telescope<T, F>(&self, ctx: &mut Ctx, f: F) -> Result<T, LoweringError>
    where
        F: FnOnce(&mut Ctx, Self::Target) -> Result<T, LoweringError>,
    {
        ctx.bind_fold(
            self.0.iter(),
            Ok(ast::Params::new()),
            |ctx, params_out, param| {
                let mut params_out = params_out?;
                let cst::Param { name, typ } = param;
                let typ_out = typ.lower_in_ctx(ctx)?;
                let param_out = ast::Param { name: name.clone(), typ: typ_out };
                params_out.push(param_out);
                Ok(params_out)
            },
            |ctx, params| f(ctx, params.map(ast::Telescope)?),
        )
    }
}

impl LowerPure for cst::Info {
    type Target = ast::Info;

    fn lower_pure(&self) -> Self::Target {
        ast::Info { span: Some(self.span) }
    }
}
