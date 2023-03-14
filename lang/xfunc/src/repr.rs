use syntax::common::*;
use syntax::ctx::{Bind, Context, LevelCtx};
use syntax::ust;

use crate::matrix;

pub trait Represent {
    fn as_data(&self) -> (ust::Data, Vec<ust::Ctor>, Vec<ust::Def>);
    fn as_codata(&self) -> (ust::Codata, Vec<ust::Dtor>, Vec<ust::Codef>);
}

impl Represent for matrix::XData {
    fn as_data(&self) -> (ust::Data, Vec<ust::Ctor>, Vec<ust::Def>) {
        let matrix::XData { name, doc, typ, ctors, dtors, exprs, .. } = self;

        let data = ust::Data {
            info: ust::Info::empty(),
            doc: doc.clone(),
            name: name.clone(),
            hidden: false,
            typ: typ.clone(),
            ctors: ctors.keys().cloned().collect(),
        };

        let defs = dtors
            .values()
            .map(|dtor| {
                let cases = ctors
                    .values()
                    .map(|ctor| {
                        let key = matrix::Key { dtor: dtor.name.clone(), ctor: ctor.name.clone() };
                        ust::Case {
                            info: ust::Info::empty(),
                            name: ctor.name.clone(),
                            args: ctor.params.instantiate(),
                            body: exprs[&key].clone(),
                        }
                    })
                    .collect();

                ust::Def {
                    info: ust::Info::empty(),
                    doc: dtor.doc.clone(),
                    name: dtor.name.clone(),
                    hidden: false,
                    params: dtor.params.clone(),
                    self_param: dtor.self_param.clone(),
                    ret_typ: dtor.ret_typ.clone(),
                    body: ust::Match { cases, info: ust::Info::empty() },
                }
            })
            .collect();

        let ctors = ctors.values().cloned().collect();

        (data, ctors, defs)
    }

    fn as_codata(&self) -> (ust::Codata, Vec<ust::Dtor>, Vec<ust::Codef>) {
        let matrix::XData { name, doc, typ, ctors, dtors, exprs, .. } = self;

        let codata = ust::Codata {
            info: ust::Info::empty(),
            doc: doc.clone(),
            name: name.clone(),
            hidden: false,
            typ: typ.clone(),
            dtors: dtors.keys().cloned().collect(),
        };

        let codefs = ctors
            .values()
            .map(|ctor| {
                let cases = dtors
                    .values()
                    .map(|dtor| {
                        let key = matrix::Key { dtor: dtor.name.clone(), ctor: ctor.name.clone() };
                        let body = &exprs[&key];
                        // Swap binding order (which is different in the matrix representation)
                        let body = body.as_ref().map(|body| {
                            let mut ctx = LevelCtx::empty();
                            ctx.bind_iter(dtor.params.params.iter(), |ctx| {
                                ctx.bind_iter(ctor.params.params.iter(), |ctx| {
                                    body.swap_with_ctx(ctx, 0, 1)
                                })
                            })
                        });
                        ust::Cocase {
                            info: ust::Info::empty(),
                            name: dtor.name.clone(),
                            params: dtor.params.instantiate(),
                            body,
                        }
                    })
                    .collect();

                ust::Codef {
                    info: ust::Info::empty(),
                    doc: ctor.doc.clone(),
                    name: ctor.name.clone(),
                    hidden: false,
                    params: ctor.params.clone(),
                    typ: ctor.typ.clone(),
                    body: ust::Comatch { cases, info: ust::Info::empty() },
                }
            })
            .collect();

        let dtors = dtors.values().cloned().collect();

        (codata, dtors, codefs)
    }
}

trait InstantiateExt {
    fn instantiate(&self) -> ust::TelescopeInst;
}

impl InstantiateExt for ust::Telescope {
    fn instantiate(&self) -> ust::TelescopeInst {
        let params = self
            .params
            .iter()
            .map(|ust::Param { name, .. }| ust::ParamInst {
                name: name.clone(),
                info: ust::Info::empty(),
                typ: (),
            })
            .collect();
        ust::TelescopeInst { params }
    }
}
