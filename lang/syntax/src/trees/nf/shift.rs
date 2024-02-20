use crate::common::*;

use super::def::*;

impl ShiftInRange for Nf {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        match self {
            Nf::TypCtor { info, name, args } => Nf::TypCtor {
                info: *info,
                name: name.clone(),
                args: args.shift_in_range(range, by),
            },
            Nf::Ctor { info, name, args } => {
                Nf::Ctor { info: *info, name: name.clone(), args: args.shift_in_range(range, by) }
            }
            Nf::Type { info } => Nf::Type { info: *info },
            Nf::Comatch { info, name, is_lambda_sugar, body } => Nf::Comatch {
                info: *info,
                name: name.clone(),
                is_lambda_sugar: *is_lambda_sugar,
                body: body.shift_in_range(range, by),
            },
            Nf::Neu { exp } => Nf::Neu { exp: exp.shift_in_range(range, by) },
        }
    }
}

impl ShiftInRange for Neu {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        match self {
            Neu::Var { info, name, idx } => {
                Neu::Var { info: *info, name: name.clone(), idx: idx.shift_in_range(range, by) }
            }
            Neu::Dtor { info, exp, name, args } => Neu::Dtor {
                info: *info,
                exp: exp.shift_in_range(range.clone(), by),
                name: name.clone(),
                args: args.shift_in_range(range, by),
            },
            Neu::Match { info, name, on_exp, body } => Neu::Match {
                info: *info,
                name: name.clone(),
                on_exp: on_exp.shift_in_range(range.clone(), by),
                body: body.shift_in_range(range, by),
            },
            Neu::Hole { info, kind } => Neu::Hole { info: *info, kind: *kind },
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
