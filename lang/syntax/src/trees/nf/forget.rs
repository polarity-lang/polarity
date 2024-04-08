use crate::ust;
use std::rc::Rc;

use super::def::*;

pub trait ForgetNF {
    type Target;

    fn forget_nf(&self) -> Self::Target;
}

impl<T: ForgetNF> ForgetNF for Rc<T> {
    type Target = Rc<T::Target>;

    fn forget_nf(&self) -> Self::Target {
        Rc::new(T::forget_nf(self))
    }
}

impl<T: ForgetNF> ForgetNF for Option<T> {
    type Target = Option<T::Target>;

    fn forget_nf(&self) -> Self::Target {
        self.as_ref().map(ForgetNF::forget_nf)
    }
}

impl<T: ForgetNF> ForgetNF for Vec<T> {
    type Target = Vec<T::Target>;

    fn forget_nf(&self) -> Self::Target {
        self.iter().map(ForgetNF::forget_nf).collect()
    }
}

impl ForgetNF for Nf {
    type Target = ust::Exp;

    fn forget_nf(&self) -> Self::Target {
        match self {
            Nf::TypCtor { info, name, args } => ust::Exp::TypCtor {
                info: *info,
                name: name.clone(),
                args: ust::Args { args: args.forget_nf() },
            },
            Nf::Ctor { info, name, args } => ust::Exp::Ctor {
                info: *info,
                name: name.clone(),
                args: ust::Args { args: args.forget_nf() },
            },
            Nf::Type { info } => ust::Exp::Type { info: *info },
            Nf::Comatch { info, name, is_lambda_sugar, body } => ust::Exp::Comatch {
                info: *info,
                ctx: (),
                name: name.clone(),
                is_lambda_sugar: *is_lambda_sugar,
                body: body.forget_nf(),
            },
            Nf::Neu { exp } => exp.forget_nf(),
        }
    }
}

impl ForgetNF for Neu {
    type Target = ust::Exp;

    fn forget_nf(&self) -> Self::Target {
        match self {
            Neu::Var { info, name, idx } => {
                ust::Exp::Var { info: *info, name: name.clone(), ctx: (), idx: *idx }
            }
            Neu::Dtor { info, exp, name, args } => ust::Exp::Dtor {
                info: *info,
                exp: exp.forget_nf(),
                name: name.clone(),
                args: ust::Args { args: args.forget_nf() },
            },
            Neu::Match { info, name, on_exp, body } => ust::Exp::Match {
                info: *info,
                ctx: (),
                name: name.clone(),
                on_exp: on_exp.forget_nf(),
                motive: None,
                ret_typ: (),
                body: body.forget_nf(),
            },
            Neu::Hole { info } => ust::Exp::Hole { info: *info },
        }
    }
}

impl ForgetNF for Match {
    type Target = ust::Match;

    fn forget_nf(&self) -> Self::Target {
        let Match { info, cases, omit_absurd } = self;

        ust::Match { info: *info, cases: cases.forget_nf(), omit_absurd: *omit_absurd }
    }
}

impl ForgetNF for Case {
    type Target = ust::Case;

    fn forget_nf(&self) -> Self::Target {
        let Case { info, name, args, body } = self;

        ust::Case { info: *info, name: name.clone(), args: args.clone(), body: body.forget_nf() }
    }
}

impl ForgetNF for TypApp {
    type Target = ust::TypApp;

    fn forget_nf(&self) -> Self::Target {
        let TypApp { info, name, args } = self;

        ust::TypApp { info: *info, name: name.clone(), args: ust::Args { args: args.forget_nf() } }
    }
}
