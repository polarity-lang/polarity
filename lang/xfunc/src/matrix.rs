use data::HashMap;
use syntax::ast;
use syntax::ast::SwapWithCtx;
use syntax::common::*;
use syntax::leveled_ctx::LeveledCtx;
use syntax::matrix;
use syntax::named::Named;

/// Take the red pill
pub fn build(prg: &ast::Prg) -> matrix::Prg {
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

impl BuildMatrix for ast::Prg {
    fn build_matrix(&self, ctx: &mut Ctx, out: &mut matrix::Prg) {
        let ast::Prg { decls, exp } = self;
        out.exp = exp.clone();

        for decl in decls.map.values() {
            match decl {
                ast::Decl::Data(data) => data.build_matrix(ctx, out),
                ast::Decl::Codata(codata) => codata.build_matrix(ctx, out),
                _ => (),
            }
        }

        for decl in decls.map.values() {
            match decl {
                ast::Decl::Ctor(ctor) => ctor.build_matrix(ctx, out),
                ast::Decl::Dtor(dtor) => dtor.build_matrix(ctx, out),
                ast::Decl::Def(def) => def.build_matrix(ctx, out),
                ast::Decl::Codef(codef) => codef.build_matrix(ctx, out),
                _ => (),
            }
        }
    }
}

impl BuildMatrix for ast::Data {
    fn build_matrix(&self, ctx: &mut Ctx, out: &mut matrix::Prg) {
        let ast::Data { info, name, typ, ctors, impl_block } = self;

        let xdata = matrix::XData {
            repr: matrix::Repr::Data,
            info: info.clone(),
            name: name.clone(),
            typ: typ.clone(),
            ctors: HashMap::default(),
            dtors: HashMap::default(),
            exprs: HashMap::default(),
            impl_block: impl_block.clone(),
        };

        for ctor in ctors {
            ctx.type_for_xtor.insert(ctor.name().clone(), name.clone());
        }

        out.map.insert(name.clone(), xdata);
    }
}

impl BuildMatrix for ast::Codata {
    fn build_matrix(&self, ctx: &mut Ctx, out: &mut matrix::Prg) {
        let ast::Codata { info, name, typ, dtors, impl_block } = self;

        let xdata = matrix::XData {
            repr: matrix::Repr::Codata,
            info: info.clone(),
            name: name.clone(),
            typ: typ.clone(),
            ctors: HashMap::default(),
            dtors: HashMap::default(),
            exprs: HashMap::default(),
            impl_block: impl_block.clone(),
        };

        for dtor in dtors {
            ctx.type_for_xtor.insert(dtor.name().clone(), name.clone());
        }

        out.map.insert(name.clone(), xdata);
    }
}

impl BuildMatrix for ast::Ctor {
    fn build_matrix(&self, ctx: &mut Ctx, out: &mut matrix::Prg) {
        let type_name = &ctx.type_for_xtor[&self.name];
        let xdata = out.map.get_mut(type_name).unwrap();
        xdata.ctors.insert(self.name.clone(), self.clone());
    }
}

impl BuildMatrix for ast::Dtor {
    fn build_matrix(&self, ctx: &mut Ctx, out: &mut matrix::Prg) {
        let type_name = &ctx.type_for_xtor[&self.name];
        let xdata = out.map.get_mut(type_name).unwrap();
        xdata.dtors.insert(self.name.clone(), self.clone());
    }
}

impl BuildMatrix for ast::Def {
    fn build_matrix(&self, _ctx: &mut Ctx, out: &mut matrix::Prg) {
        let type_name = &self.on_typ.name;
        let xdata = out.map.get_mut(type_name).unwrap();
        xdata.dtors.insert(self.name.clone(), self.to_dtor());

        let ast::Match { cases, .. } = &self.body;

        for case in cases {
            let ast::Case { name, body, .. } = case;
            let key = matrix::Key { dtor: self.name.clone(), ctor: name.clone() };
            xdata.exprs.insert(key, body.clone());
        }
    }
}

impl BuildMatrix for ast::Codef {
    fn build_matrix(&self, _ctx: &mut Ctx, out: &mut matrix::Prg) {
        let type_name = &self.typ.name;
        let xdata = out.map.get_mut(type_name).unwrap();
        xdata.ctors.insert(self.name.clone(), self.to_ctor());

        let ast::Comatch { cases, .. } = &self.body;

        for case in cases {
            let ast::Cocase { name, body, .. } = case;
            let key = matrix::Key { ctor: self.name.clone(), dtor: name.clone() };
            // Swap binding order to the order imposed by the matrix representation
            let body = body.as_ref().map(|body| {
                let mut ctx = LeveledCtx::empty();
                ctx.bind(self.params.params.iter(), |ctx| {
                    ctx.bind(case.args.params.iter(), |ctx| body.swap_with_ctx(ctx, 0, 1))
                })
            });
            xdata.exprs.insert(key, body.clone());
        }
    }
}
