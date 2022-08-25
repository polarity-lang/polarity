use std::rc::Rc;

use syntax::ast;
use syntax::cst;
use syntax::cst::Telescope;
use syntax::named::Named;

use super::ctx::*;
use super::result::*;
use super::types::*;

pub fn lower(prg: cst::Prg) -> Result<ast::Prg, LoweringError> {
    let cst::Prg { decls, exp } = prg;
    let mut ctx = Ctx::empty();

    decls.lower_in_ctx(&mut ctx)?;
    let exp = exp.lower_in_ctx(&mut ctx)?;

    Ok(ast::Prg { decls: ctx.decls, exp })
}

impl Lower for cst::Decl {
    type Target = ();

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        ctx.add_name(self.name())?;
        let decl = match self {
            cst::Decl::Data(data) => ast::Decl::Data(data.lower_in_ctx(ctx)?),
            cst::Decl::Codata(codata) => ast::Decl::Codata(codata.lower_in_ctx(ctx)?),
            cst::Decl::Def(def) => ast::Decl::Def(def.lower_in_ctx(ctx)?),
            cst::Decl::Codef(codef) => ast::Decl::Codef(codef.lower_in_ctx(ctx)?),
        };
        ctx.add_decl(decl)?;
        Ok(())
    }
}

impl Lower for cst::Data {
    type Target = ast::Data;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::Data { name, params, ctors } = self;

        Ok(ast::Data {
            name: name.clone(),
            params: params.lower_in_ctx(ctx)?,
            ctors: ctors.lower_in_ctx(ctx)?,
        })
    }
}

impl Lower for cst::Codata {
    type Target = ast::Codata;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::Codata { name, params, ctors } = self;

        Ok(ast::Codata {
            name: name.clone(),
            params: params.lower_in_ctx(ctx)?,
            ctors: ctors.lower_in_ctx(ctx)?,
        })
    }
}

impl Lower for cst::Ctor {
    type Target = ast::Ctor;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::Ctor { name, params, typ } = self;

        ctx.add_name(name)?;

        params.lower_telescope(ctx, |ctx, params| {
            Ok(ast::Ctor { name: name.clone(), params, typ: typ.lower_in_ctx(ctx)? })
        })
    }
}

impl Lower for cst::Dtor {
    type Target = ast::Dtor;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::Dtor { name, params, on_typ, in_typ } = self;

        ctx.add_name(name)?;

        params.lower_telescope(ctx, |ctx, params| {
            Ok(ast::Dtor {
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
        let cst::Def { name, params, on_typ, in_typ, body } = self;

        params.lower_telescope(ctx, |ctx, params| {
            Ok(ast::Def {
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
        let cst::Codef { name, params, typ, body } = self;

        params.lower_telescope(ctx, |ctx, params| {
            Ok(ast::Codef {
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
        let cst::Match { cases } = self;

        Ok(ast::Match { cases: cases.lower_in_ctx(ctx)? })
    }
}

impl Lower for cst::Comatch {
    type Target = ast::Comatch;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::Comatch { cases } = self;

        Ok(ast::Comatch { cases: cases.lower_in_ctx(ctx)? })
    }
}

impl Lower for cst::Case {
    type Target = ast::Case;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::Case { name, args, body } = self;

        args.lower_telescope(ctx, |ctx, args| {
            Ok(ast::Case { name: name.clone(), args, body: body.lower_in_ctx(ctx)? })
        })
    }
}

impl Lower for cst::TypApp {
    type Target = ast::TypApp;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        let cst::TypApp { name, subst } = self;

        Ok(ast::TypApp { name: name.clone(), subst: subst.lower_in_ctx(ctx)? })
    }
}

impl Lower for cst::Exp {
    type Target = ast::Exp;

    fn lower_in_ctx(&self, ctx: &mut Ctx) -> Result<Self::Target, LoweringError> {
        match self {
            cst::Exp::Var { name } => Ok(ast::Exp::Var { var: ctx.lookup(name)? }),
            cst::Exp::Ctor { name, subst } => {
                Ok(ast::Exp::Ctor { name: name.clone(), subst: subst.lower_in_ctx(ctx)? })
            }
            cst::Exp::Dtor { exp, name, subst } => Ok(ast::Exp::Dtor {
                exp: exp.lower_in_ctx(ctx)?,
                name: name.clone(),
                subst: subst.lower_in_ctx(ctx)?,
            }),
            cst::Exp::Ano { exp, typ } => {
                Ok(ast::Exp::Ano { exp: exp.lower_in_ctx(ctx)?, typ: typ.lower_in_ctx(ctx)? })
            }
            cst::Exp::Type => Ok(ast::Exp::Type),
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
        Ok(Rc::new((&**self).lower_in_ctx(ctx)?))
    }
}

impl LowerTelescope for Telescope {
    type Target = ast::Telescope;

    /// Lower a telescope
    ///
    /// Execute a function `f` under the context where all binders
    /// of the telescope are in scope.
    fn lower_telescope<T, F>(&self, ctx: &mut Ctx, f: F) -> Result<T, LoweringError>
    where
        F: Fn(&mut Ctx, Self::Target) -> Result<T, LoweringError>,
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
