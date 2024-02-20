use crate::common::*;
use crate::ust;

use super::def::*;

impl Forget for Nf {
    type Target = ust::Exp;

    fn forget(&self) -> Self::Target {
        match self {
            Nf::TypCtor { info, name, args } => ust::Exp::TypCtor {
                info: *info,
                name: name.clone(),
                args: ust::Args { args: args.forget() },
            },
            Nf::Ctor { info, name, args } => ust::Exp::Ctor {
                info: *info,
                name: name.clone(),
                args: ust::Args { args: args.forget() },
            },
            Nf::Type { info } => ust::Exp::Type { info: *info },
            Nf::Comatch { info, name, is_lambda_sugar, body } => ust::Exp::Comatch {
                info: *info,
                ctx: (),
                name: name.clone(),
                is_lambda_sugar: *is_lambda_sugar,
                body: body.forget(),
            },
            Nf::Neu { exp } => exp.forget(),
        }
    }
}

impl Forget for Neu {
    type Target = ust::Exp;

    fn forget(&self) -> Self::Target {
        match self {
            Neu::Var { info, name, idx } => {
                ust::Exp::Var { info: *info, name: name.clone(), ctx: (), idx: *idx }
            }
            Neu::Dtor { info, exp, name, args } => ust::Exp::Dtor {
                info: *info,
                exp: exp.forget(),
                name: name.clone(),
                args: ust::Args { args: args.forget() },
            },
            Neu::Match { info, name, on_exp, body } => ust::Exp::Match {
                info: *info,
                ctx: (),
                name: name.clone(),
                on_exp: on_exp.forget(),
                motive: None,
                ret_typ: (),
                body: body.forget(),
            },
            Neu::Hole { info } => ust::Exp::Hole { info: *info },
        }
    }
}

impl Forget for Match {
    type Target = ust::Match;

    fn forget(&self) -> Self::Target {
        let Match { info, cases, omit_absurd } = self;

        ust::Match { info: *info, cases: cases.forget(), omit_absurd: *omit_absurd }
    }
}

impl Forget for Case {
    type Target = ust::Case;

    fn forget(&self) -> Self::Target {
        let Case { info, name, args, body } = self;

        ust::Case { info: *info, name: name.clone(), args: args.clone(), body: body.forget() }
    }
}

impl Forget for TypApp {
    type Target = ust::TypApp;

    fn forget(&self) -> Self::Target {
        let TypApp { info, name, args } = self;

        ust::TypApp { info: *info, name: name.clone(), args: ust::Args { args: args.forget() } }
    }
}
