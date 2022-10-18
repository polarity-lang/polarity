use syntax::ast::SwapWithCtx;
use syntax::leveled_ctx::LeveledCtx;
use syntax::matrix;
use syntax::ust;

pub trait Represent {
    fn as_data(&self) -> (ust::Data, Vec<ust::Ctor>, Vec<ust::Def>);
    fn as_codata(&self) -> (ust::Codata, Vec<ust::Dtor>, Vec<ust::Codef>);
}

impl Represent for matrix::XData {
    fn as_data(&self) -> (ust::Data, Vec<ust::Ctor>, Vec<ust::Def>) {
        let matrix::XData { name, typ, ctors, dtors, exprs, impl_block, .. } = self;

        let data = ust::Data {
            info: ust::Info::empty(),
            name: name.clone(),
            typ: typ.clone(),
            ctors: ctors.keys().cloned().collect(),
            impl_block: impl_block.clone(),
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
                            args: ctor.params.clone(),
                            body: exprs[&key].clone(),
                        }
                    })
                    .collect();

                ust::Def {
                    info: ust::Info::empty(),
                    name: dtor.name.clone(),
                    params: dtor.params.clone(),
                    on_typ: dtor.on_typ.clone(),
                    in_typ: dtor.in_typ.clone(),
                    body: ust::Match { cases, info: ust::Info::empty() },
                }
            })
            .collect();

        let ctors = ctors.values().cloned().collect();

        (data, ctors, defs)
    }

    fn as_codata(&self) -> (ust::Codata, Vec<ust::Dtor>, Vec<ust::Codef>) {
        let matrix::XData { name, typ, ctors, dtors, exprs, impl_block, .. } = self;

        let codata = ust::Codata {
            info: ust::Info::empty(),
            name: name.clone(),
            typ: typ.clone(),
            dtors: dtors.keys().cloned().collect(),
            impl_block: impl_block.clone(),
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
                            let mut ctx = LeveledCtx::empty();
                            ctx.bind(dtor.params.params.iter(), |ctx| {
                                ctx.bind(ctor.params.params.iter(), |ctx| {
                                    body.swap_with_ctx(ctx, 0, 1)
                                })
                            })
                        });
                        ust::Cocase {
                            info: ust::Info::empty(),
                            name: dtor.name.clone(),
                            args: dtor.params.clone(),
                            body,
                        }
                    })
                    .collect();

                ust::Codef {
                    info: ust::Info::empty(),
                    name: ctor.name.clone(),
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
