use std::rc::Rc;

use syntax::elab::*;

pub trait Collector {
    fn add(&mut self, info: &Info);
    fn add_typed(&mut self, info: &TypedInfo);
}

pub trait Collect {
    fn collect<C: Collector>(&self, c: &mut C);
}

impl Collect for Prg {
    fn collect<C: Collector>(&self, c: &mut C) {
        let Prg { decls, exp } = self;
        decls.collect(c);
        exp.collect(c);
    }
}

impl Collect for Decls {
    fn collect<C: Collector>(&self, c: &mut C) {
        let Decls { map, order: _ } = self;

        for decl in map.values() {
            decl.collect(c);
        }
    }
}

impl Collect for Decl {
    fn collect<C: Collector>(&self, c: &mut C) {
        match self {
            Decl::Data(data) => data.collect(c),
            Decl::Codata(codata) => codata.collect(c),
            Decl::Ctor(ctor) => ctor.collect(c),
            Decl::Dtor(dtor) => dtor.collect(c),
            Decl::Def(def) => def.collect(c),
            Decl::Codef(codef) => codef.collect(c),
        }
    }
}

impl Collect for Data {
    fn collect<C: Collector>(&self, c: &mut C) {
        let Data { info, name: _, typ, ctors: _, impl_block: _ } = self;
        c.add(info);
        typ.collect(c);
    }
}

impl Collect for Codata {
    fn collect<C: Collector>(&self, c: &mut C) {
        let Codata { info, name: _, typ, dtors: _, impl_block: _ } = self;

        c.add(info);
        typ.collect(c);
    }
}

impl Collect for Ctor {
    fn collect<C: Collector>(&self, c: &mut C) {
        let Ctor { info, name: _, params, typ } = self;

        c.add(info);
        params.collect(c);
        typ.collect(c);
    }
}

impl Collect for Dtor {
    fn collect<C: Collector>(&self, c: &mut C) {
        let Dtor { info, name: _, params, on_typ, in_typ } = self;

        c.add(info);
        params.collect(c);
        on_typ.collect(c);
        in_typ.collect(c);
    }
}

impl Collect for Def {
    fn collect<C: Collector>(&self, c: &mut C) {
        let Def { info, name: _, params, on_typ, in_typ, body } = self;

        c.add(info);
        params.collect(c);
        on_typ.collect(c);
        in_typ.collect(c);
        body.collect(c);
    }
}

impl Collect for Codef {
    fn collect<C: Collector>(&self, c: &mut C) {
        let Codef { info, name: _, params, typ, body } = self;

        c.add(info);
        params.collect(c);
        typ.collect(c);
        body.collect(c);
    }
}

impl Collect for Exp {
    fn collect<C: Collector>(&self, c: &mut C) {
        match self {
            Exp::Var { info, idx: _ } => c.add_typed(info),
            Exp::TyCtor { info, name: _, args } => {
                c.add_typed(info);
                args.collect(c)
            }
            Exp::Ctor { info, name: _, args } => {
                c.add_typed(info);
                args.collect(c);
            }
            Exp::Dtor { info, exp, name: _, args } => {
                c.add_typed(info);
                exp.collect(c);
                args.collect(c);
            }
            Exp::Anno { info, exp, typ } => {
                c.add_typed(info);
                exp.collect(c);
                typ.collect(c);
            }
            Exp::Type { info } => {
                c.add_typed(info);
            }
        }
    }
}

impl Collect for Match {
    fn collect<C: Collector>(&self, c: &mut C) {
        let Match { info, cases } = self;
        c.add(info);
        for case in cases {
            case.collect(c);
        }
    }
}

impl Collect for Comatch {
    fn collect<C: Collector>(&self, c: &mut C) {
        let Comatch { info, cases } = self;
        c.add(info);
        for case in cases {
            case.collect(c);
        }
    }
}

impl Collect for Case {
    fn collect<C: Collector>(&self, c: &mut C) {
        let Case { info, name: _, args, eqns, body } = self;

        c.add(info);
        args.collect(c);
        eqns.collect(c);
        body.collect(c);
    }
}

impl Collect for Cocase {
    fn collect<C: Collector>(&self, c: &mut C) {
        let Cocase { info, name: _, args, eqns, body } = self;

        c.add(info);
        args.collect(c);
        eqns.collect(c);
        body.collect(c);
    }
}

impl Collect for Eqns {
    fn collect<C: Collector>(&self, c: &mut C) {
        let Eqns { eqns, params } = self;

        eqns.collect(c);
        params.collect(c);
    }
}

impl Collect for EqnParam {
    fn collect<C: Collector>(&self, c: &mut C) {
        let EqnParam { name: _, eqn } = self;

        eqn.collect(c);
    }
}

impl Collect for Eqn {
    fn collect<C: Collector>(&self, c: &mut C) {
        let Eqn { info, lhs, rhs } = self;

        c.add(info);
        lhs.collect(c);
        rhs.collect(c);
    }
}

impl Collect for TypAbs {
    fn collect<C: Collector>(&self, c: &mut C) {
        let TypAbs { params } = self;

        params.collect(c);
    }
}

impl Collect for TypApp {
    fn collect<C: Collector>(&self, c: &mut C) {
        let TypApp { info, name: _, args } = self;

        c.add(info);
        args.collect(c);
    }
}

impl Collect for Telescope {
    fn collect<C: Collector>(&self, c: &mut C) {
        let Telescope(params) = self;

        params.collect(c);
    }
}

impl Collect for Param {
    fn collect<C: Collector>(&self, c: &mut C) {
        let Param { name: _, typ } = self;

        typ.collect(c);
    }
}

impl<T: Collect> Collect for Option<T> {
    fn collect<C: Collector>(&self, c: &mut C) {
        if let Some(x) = self {
            x.collect(c);
        }
    }
}

impl<T: Collect> Collect for Rc<T> {
    fn collect<C: Collector>(&self, c: &mut C) {
        T::collect(self, c);
    }
}

impl<T: Collect> Collect for Vec<T> {
    fn collect<C: Collector>(&self, c: &mut C) {
        for x in self {
            x.collect(c);
        }
    }
}
