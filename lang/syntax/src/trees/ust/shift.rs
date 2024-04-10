use crate::common::*;

use super::def::*;

impl ShiftInRange for Exp {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        match self {
            Exp::Var { info, span, name, ctx: (), idx } => Exp::Var {
                info: *info,
                span: span.clone(),
                name: name.clone(),
                ctx: (),
                idx: idx.shift_in_range(range, by),
            },
            Exp::TypCtor { info, span, name, args } => Exp::TypCtor {
                info: *info,
                span: span.clone(),
                name: name.clone(),
                args: args.shift_in_range(range, by),
            },
            Exp::Ctor { info, span, name, args } => Exp::Ctor {
                info: *info,
                span: span.clone(),
                name: name.clone(),
                args: args.shift_in_range(range, by),
            },
            Exp::Dtor { info, span, exp, name, args } => Exp::Dtor {
                info: *info,
                span: span.clone(),
                exp: exp.shift_in_range(range.clone(), by),
                name: name.clone(),
                args: args.shift_in_range(range, by),
            },
            Exp::Anno { info, span, exp, typ } => Exp::Anno {
                info: *info,
                span: span.clone(),
                exp: exp.shift_in_range(range.clone(), by),
                typ: typ.shift_in_range(range, by),
            },
            Exp::Type { info, span } => Exp::Type { info: *info, span: span.clone() },
            Exp::Match { info, span, ctx: (), name, on_exp, motive, ret_typ: (), body } => {
                Exp::Match {
                    info: *info,
                    span: span.clone(),
                    ctx: (),
                    name: name.clone(),
                    on_exp: on_exp.shift_in_range(range.clone(), by),
                    motive: motive.shift_in_range(range.clone(), by),
                    ret_typ: (),
                    body: body.shift_in_range(range, by),
                }
            }
            Exp::Comatch { info, span, ctx: (), name, is_lambda_sugar, body } => Exp::Comatch {
                info: *info,
                span: span.clone(),
                ctx: (),
                name: name.clone(),
                is_lambda_sugar: *is_lambda_sugar,
                body: body.shift_in_range(range, by),
            },
            Exp::Hole { info, span } => Exp::Hole { info: *info, span: span.clone() },
        }
    }
}

impl ShiftInRange for Motive {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Motive { span, param, ret_typ } = self;

        Motive {
            span: span.clone(),
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
        let TypApp { info, span, name, args } = self;
        TypApp {
            info: *info,
            span: span.clone(),
            name: name.clone(),
            args: args.shift_in_range(range, by),
        }
    }
}

impl ShiftInRange for Args {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        Self { args: self.args.shift_in_range(range, by) }
    }
}
