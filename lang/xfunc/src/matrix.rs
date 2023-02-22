use data::HashMap;
use syntax::common::*;
use syntax::ctx::{Bind, Context, LevelCtx};
use syntax::matrix;
use syntax::ust;

/// Take the red pill
pub fn build(prg: &ust::Prg) -> matrix::Prg {
    let mut out = matrix::Prg { map: HashMap::default(), exp: None };
    let mut ctx = Ctx::empty();
    prg.build_matrix(&mut ctx, &mut out);
    out
}

pub trait BuildMatrix {
    fn build_matrix(&self, ctx: &mut Ctx, out: &mut matrix::Prg);
}

pub struct Ctx {
    type_for_xtor: HashMap<Ident, Ident>,
}

impl Ctx {
    pub fn empty() -> Self {
        Self { type_for_xtor: HashMap::default() }
    }
}

impl BuildMatrix for ust::Prg {
    fn build_matrix(&self, ctx: &mut Ctx, out: &mut matrix::Prg) {
        let ust::Prg { decls, exp } = self;
        out.exp = exp.clone();

        for decl in decls.map.values() {
            match decl {
                ust::Decl::Data(data) => data.build_matrix(ctx, out),
                ust::Decl::Codata(codata) => codata.build_matrix(ctx, out),
                _ => (),
            }
        }

        for decl in decls.map.values() {
            match decl {
                ust::Decl::Ctor(ctor) => ctor.build_matrix(ctx, out),
                ust::Decl::Dtor(dtor) => dtor.build_matrix(ctx, out),
                ust::Decl::Def(def) => def.build_matrix(ctx, out),
                ust::Decl::Codef(codef) => codef.build_matrix(ctx, out),
                _ => (),
            }
        }
    }
}

impl BuildMatrix for ust::Data {
    fn build_matrix(&self, ctx: &mut Ctx, out: &mut matrix::Prg) {
        let ust::Data { info, doc: _, name, hidden: _, typ, ctors } = self;

        let xdata = matrix::XData {
            repr: matrix::Repr::Data,
            info: info.clone(),
            name: name.clone(),
            typ: typ.clone(),
            ctors: HashMap::default(),
            dtors: HashMap::default(),
            exprs: HashMap::default(),
        };

        for ctor in ctors {
            ctx.type_for_xtor.insert(ctor.name().clone(), name.clone());
        }

        out.map.insert(name.clone(), xdata);
    }
}

impl BuildMatrix for ust::Codata {
    fn build_matrix(&self, ctx: &mut Ctx, out: &mut matrix::Prg) {
        let ust::Codata { info, doc: _, name, hidden: _, typ, dtors } = self;

        let xdata = matrix::XData {
            repr: matrix::Repr::Codata,
            info: info.clone(),
            name: name.clone(),
            typ: typ.clone(),
            ctors: HashMap::default(),
            dtors: HashMap::default(),
            exprs: HashMap::default(),
        };

        for dtor in dtors {
            ctx.type_for_xtor.insert(dtor.name().clone(), name.clone());
        }

        out.map.insert(name.clone(), xdata);
    }
}

impl BuildMatrix for ust::Ctor {
    fn build_matrix(&self, ctx: &mut Ctx, out: &mut matrix::Prg) {
        let type_name = &ctx.type_for_xtor[&self.name];
        let xdata = out.map.get_mut(type_name).unwrap();
        xdata.ctors.insert(self.name.clone(), self.clone());
    }
}

impl BuildMatrix for ust::Dtor {
    fn build_matrix(&self, ctx: &mut Ctx, out: &mut matrix::Prg) {
        let type_name = &ctx.type_for_xtor[&self.name];
        let xdata = out.map.get_mut(type_name).unwrap();
        xdata.dtors.insert(self.name.clone(), self.clone());
    }
}

impl BuildMatrix for ust::Def {
    fn build_matrix(&self, _ctx: &mut Ctx, out: &mut matrix::Prg) {
        let type_name = &self.self_param.typ.name;
        let xdata = out.map.get_mut(type_name).unwrap();
        xdata.dtors.insert(self.name.clone(), self.to_dtor());

        let ust::Match { cases, .. } = &self.body;

        for case in cases {
            let ust::Case { name, body, .. } = case;
            let key = matrix::Key { dtor: self.name.clone(), ctor: name.clone() };
            xdata.exprs.insert(key, body.clone());
        }
    }
}

impl BuildMatrix for ust::Codef {
    fn build_matrix(&self, _ctx: &mut Ctx, out: &mut matrix::Prg) {
        let type_name = &self.typ.name;
        let xdata = out.map.get_mut(type_name).unwrap();
        xdata.ctors.insert(self.name.clone(), self.to_ctor());

        let ust::Comatch { cases, .. } = &self.body;

        for case in cases {
            let ust::Cocase { name, body, .. } = case;
            let key = matrix::Key { ctor: self.name.clone(), dtor: name.clone() };
            // Swap binding order to the order imposed by the matrix representation
            let body = body.as_ref().map(|body| {
                let mut ctx = LevelCtx::empty();
                // TODO: Reconsider where to swap this
                ctx.bind_iter(self.params.params.iter().map(|_| ()), |ctx| {
                    ctx.bind_iter(case.params.params.iter().map(|_| ()), |ctx| {
                        body.swap_with_ctx(ctx, 0, 1)
                    })
                })
            });
            xdata.exprs.insert(key, body.clone());
        }
    }
}
