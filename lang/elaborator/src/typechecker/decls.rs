use std::rc::Rc;

use miette_util::ToMiette;
use syntax::generic::*;
use tracer::trace;

use crate::normalizer::{env::ToEnv, normalize::Normalize};

use super::{
    ctx::Ctx,
    typecheck::{Check, Infer, InferTelescope, WithDestructee, WithScrutinee},
    util::ExpectTypApp,
    TypeError,
};

pub fn check(prg: &Prg) -> Result<Prg, TypeError> {
    let mut var_ctx = Ctx::default();
    prg.check_wf(prg, &mut var_ctx)
}

pub trait CheckToplevel {
    type Target;
    fn check_wf(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError>;
}

impl<T: CheckToplevel> CheckToplevel for Rc<T> {
    type Target = Rc<T::Target>;

    fn check_wf(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        Ok(Rc::new((**self).check_wf(prg, ctx)?))
    }
}

impl CheckToplevel for Prg {
    type Target = Prg;

    fn check_wf(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let Prg { decls } = self;

        let decls_out = decls.check_wf(prg, ctx)?;

        Ok(Prg { decls: decls_out })
    }
}

/// Check all declarations in a program
impl CheckToplevel for Decls {
    type Target = Decls;

    fn check_wf(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let Decls { map, lookup_table } = self;

        // FIXME: Reconsider order

        let map_out = map
            .iter()
            .map(|(name, decl)| Ok((name.clone(), decl.check_wf(prg, ctx)?)))
            .collect::<Result<_, TypeError>>()?;

        Ok(Decls { map: map_out, lookup_table: lookup_table.clone() })
    }
}

/// Check a declaration
impl CheckToplevel for Decl {
    type Target = Decl;

    #[trace("{:P} |- {} =>", ctx, self.name())]
    fn check_wf(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let out = match self {
            Decl::Data(data) => Decl::Data(data.check_wf(prg, ctx)?),
            Decl::Codata(codata) => Decl::Codata(codata.check_wf(prg, ctx)?),
            Decl::Ctor(ctor) => Decl::Ctor(ctor.check_wf(prg, ctx)?),
            Decl::Dtor(dtor) => Decl::Dtor(dtor.check_wf(prg, ctx)?),
            Decl::Def(def) => Decl::Def(def.check_wf(prg, ctx)?),
            Decl::Codef(codef) => Decl::Codef(codef.check_wf(prg, ctx)?),
            Decl::Let(tl_let) => Decl::Let(tl_let.check_wf(prg, ctx)?),
        };
        Ok(out)
    }
}

/// Check a data declaration
impl CheckToplevel for Data {
    type Target = Data;

    fn check_wf(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let Data { span, doc, name, attr, typ, ctors } = self;

        let typ_out = typ.check_wf(prg, ctx)?;

        Ok(Data {
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            attr: attr.clone(),
            typ: typ_out,
            ctors: ctors.clone(),
        })
    }
}

/// Infer a codata declaration
impl CheckToplevel for Codata {
    type Target = Codata;

    fn check_wf(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let Codata { span, doc, name, attr, typ, dtors } = self;

        let typ_out = typ.check_wf(prg, ctx)?;

        Ok(Codata {
            span: *span,
            doc: doc.clone(),
            name: name.clone(),
            attr: attr.clone(),
            typ: typ_out,
            dtors: dtors.clone(),
        })
    }
}

/// Infer a codata declaration
impl CheckToplevel for TypAbs {
    type Target = TypAbs;

    fn check_wf(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let TypAbs { params } = self;

        params.infer_telescope(prg, ctx, |_, params_out| Ok(TypAbs { params: params_out }))
    }
}

/// Infer a constructor declaration
impl CheckToplevel for Ctor {
    type Target = Ctor;

    fn check_wf(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let Ctor { span, doc, name, params, typ } = self;

        // Check that the constructor lies in the data type it is defined in
        let data_type = prg.decls.data_for_ctor(name, *span)?;
        let expected = &data_type.name;
        if &typ.name != expected {
            return Err(TypeError::NotInType {
                expected: expected.clone(),
                actual: typ.name.clone(),
                span: typ.span.to_miette(),
            });
        }

        params.infer_telescope(prg, ctx, |ctx, params_out| {
            let typ_out = typ.infer(prg, ctx)?;

            Ok(Ctor {
                span: *span,
                doc: doc.clone(),
                name: name.clone(),
                params: params_out,
                typ: typ_out,
            })
        })
    }
}

/// Infer a destructor declaration
impl CheckToplevel for Dtor {
    type Target = Dtor;

    fn check_wf(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let Dtor { span, doc, name, params, self_param, ret_typ } = self;

        // Check that the destructor lies in the codata type it is defined in
        let codata_type = prg.decls.codata_for_dtor(name, *span)?;
        let expected = &codata_type.name;
        if &self_param.typ.name != expected {
            return Err(TypeError::NotInType {
                expected: expected.clone(),
                actual: self_param.typ.name.clone(),
                span: self_param.typ.span.to_miette(),
            });
        }

        params.infer_telescope(prg, ctx, |ctx, params_out| {
            self_param.infer_telescope(prg, ctx, |ctx, self_param_out| {
                let ret_typ_out = ret_typ.infer(prg, ctx)?;

                Ok(Dtor {
                    span: *span,
                    doc: doc.clone(),
                    name: name.clone(),
                    params: params_out,
                    self_param: self_param_out,
                    ret_typ: ret_typ_out,
                })
            })
        })
    }
}

/// Infer a definition
impl CheckToplevel for Def {
    type Target = Def;

    fn check_wf(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let Def { span, doc, name, attr, params, self_param, ret_typ, body } = self;

        params.infer_telescope(prg, ctx, |ctx, params_out| {
            let self_param_nf = self_param.typ.normalize(prg, &mut ctx.env())?;

            let (ret_typ_out, ret_typ_nf, self_param_out) =
                self_param.infer_telescope(prg, ctx, |ctx, self_param_out| {
                    let ret_typ_out = ret_typ.infer(prg, ctx)?;
                    let ret_typ_nf = ret_typ.normalize(prg, &mut ctx.env())?;
                    Ok((ret_typ_out, ret_typ_nf, self_param_out))
                })?;

            let body_out =
                WithScrutinee { inner: body, scrutinee: self_param_nf.expect_typ_app()? }
                    .check_ws(prg, ctx, ret_typ_nf)?;
            Ok(Def {
                span: *span,
                doc: doc.clone(),
                name: name.clone(),
                attr: attr.clone(),
                params: params_out,
                self_param: self_param_out,
                ret_typ: ret_typ_out,
                body: body_out,
            })
        })
    }
}

/// Infer a co-definition
impl CheckToplevel for Codef {
    type Target = Codef;

    fn check_wf(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let Codef { span, doc, name, attr, params, typ, body } = self;

        params.infer_telescope(prg, ctx, |ctx, params_out| {
            let typ_out = typ.infer(prg, ctx)?;
            let typ_nf = typ.normalize(prg, &mut ctx.env())?;
            let wd = WithDestructee {
                inner: body,
                label: Some(name.to_owned()),
                n_label_args: params.len(),
                destructee: typ_nf.expect_typ_app()?,
            };
            let body_out = wd.infer_wd(prg, ctx)?;
            Ok(Codef {
                span: *span,
                doc: doc.clone(),
                name: name.clone(),
                attr: attr.clone(),
                params: params_out,
                typ: typ_out,
                body: body_out,
            })
        })
    }
}

impl CheckToplevel for Let {
    type Target = Let;

    fn check_wf(&self, prg: &Prg, ctx: &mut Ctx) -> Result<Self::Target, TypeError> {
        let Let { span, doc, name, attr, params, typ, body } = self;

        params.infer_telescope(prg, ctx, |ctx, params_out| {
            let typ_out = typ.infer(prg, ctx)?;
            let typ_nf = typ.normalize(prg, &mut ctx.env())?;
            let body_out = body.check(prg, ctx, typ_nf)?;

            Ok(Let {
                span: *span,
                doc: doc.clone(),
                name: name.clone(),
                attr: attr.clone(),
                params: params_out,
                typ: typ_out,
                body: body_out,
            })
        })
    }
}
