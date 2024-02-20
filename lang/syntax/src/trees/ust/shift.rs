use crate::common::*;

use super::def::*;

impl ShiftInRange for Exp {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        match self {
            Exp::Var { info, name, ctx: (), idx } => Exp::Var {
                info: *info,
                name: name.clone(),
                ctx: (),
                idx: idx.shift_in_range(range, by),
            },
            Exp::TypCtor { info, name, args } => Exp::TypCtor {
                info: *info,
                name: name.clone(),
                args: args.shift_in_range(range, by),
            },
            Exp::Ctor { info, name, args } => {
                Exp::Ctor { info: *info, name: name.clone(), args: args.shift_in_range(range, by) }
            }
            Exp::Dtor { info, exp, name, args } => Exp::Dtor {
                info: *info,
                exp: exp.shift_in_range(range.clone(), by),
                name: name.clone(),
                args: args.shift_in_range(range, by),
            },
            Exp::Anno { info, exp, typ } => Exp::Anno {
                info: *info,
                exp: exp.shift_in_range(range.clone(), by),
                typ: typ.shift_in_range(range, by),
            },
            Exp::Type { info } => Exp::Type { info: *info },
            Exp::Match { info, ctx: (), name, on_exp, motive, ret_typ: (), body } => Exp::Match {
                info: *info,
                ctx: (),
                name: name.clone(),
                on_exp: on_exp.shift_in_range(range.clone(), by),
                motive: motive.shift_in_range(range.clone(), by),
                ret_typ: (),
                body: body.shift_in_range(range, by),
            },
            Exp::Comatch { info, ctx: (), name, is_lambda_sugar, body } => Exp::Comatch {
                info: *info,
                ctx: (),
                name: name.clone(),
                is_lambda_sugar: *is_lambda_sugar,
                body: body.shift_in_range(range, by),
            },
            Exp::Hole { info } => Exp::Hole { info: *info },
        }
    }
}

impl ShiftInRange for Motive {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Motive { info, param, ret_typ } = self;

        Motive {
            info: *info,
            param: param.clone(),
            ret_typ: ret_typ.shift_in_range(range.shift(1), by),
        }
    }
}

impl ShiftInRange for Match {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Match { info, cases, omit_absurd } = self;
        Match { info: *info, cases: cases.shift_in_range(range, by), omit_absurd: *omit_absurd }
    }
}

impl ShiftInRange for Case {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Case { info, name, args, body } = self;
        Case {
            info: *info,
            name: name.clone(),
            args: args.clone(),
            body: body.shift_in_range(range.shift(1), by),
        }
    }
}

impl ShiftInRange for TypApp {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let TypApp { info, name, args } = self;
        TypApp { info: *info, name: name.clone(), args: args.shift_in_range(range, by) }
    }
}

impl ShiftInRange for Args {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        Self { args: self.args.shift_in_range(range, by) }
    }
}
