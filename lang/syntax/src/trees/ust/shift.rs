use crate::common::*;

use super::def::*;

impl Shift for Exp {
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
            Exp::LocalMatch { info, ctx: (), name, on_exp, motive, ret_typ: (), body } => {
                Exp::LocalMatch {
                    info: *info,
                    ctx: (),
                    name: name.clone(),
                    on_exp: on_exp.shift_in_range(range.clone(), by),
                    motive: motive.shift_in_range(range.clone(), by),
                    ret_typ: (),
                    body: body.shift_in_range(range, by),
                }
            }
            Exp::LocalComatch { info, ctx: (), name, is_lambda_sugar, body } => Exp::LocalComatch {
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

impl Shift for Motive {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Motive { info, param, ret_typ } = self;

        Motive {
            info: *info,
            param: param.clone(),
            ret_typ: ret_typ.shift_in_range(range.shift(1), by),
        }
    }
}

impl Shift for Match {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Match { info, cases, omit_absurd } = self;
        Match { info: *info, cases: cases.shift_in_range(range, by), omit_absurd: *omit_absurd }
    }
}

impl Shift for Case {
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

impl Shift for TypApp {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let TypApp { info, name, args } = self;
        TypApp { info: *info, name: name.clone(), args: args.shift_in_range(range, by) }
    }
}

impl Shift for Args {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        Self { args: self.args.shift_in_range(range, by) }
    }
}
