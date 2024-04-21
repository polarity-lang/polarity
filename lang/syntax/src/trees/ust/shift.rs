use crate::common::*;

use super::def::*;

impl Shift for Exp {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        match self {
            Exp::Variable(e) => Exp::Variable(e.shift_in_range(range, by)),
            Exp::TypCtor(e) => Exp::TypCtor(e.shift_in_range(range, by)),
            Exp::Call(e) => Exp::Call(e.shift_in_range(range, by)),
            Exp::DotCall(e) => Exp::DotCall(e.shift_in_range(range, by)),
            Exp::Anno(e) => Exp::Anno(e.shift_in_range(range, by)),
            Exp::Type(e) => Exp::Type(e.shift_in_range(range, by)),
            Exp::LocalMatch(e) => Exp::LocalMatch(e.shift_in_range(range, by)),
            Exp::LocalComatch(e) => Exp::LocalComatch(e.shift_in_range(range, by)),
            Exp::Hole(e) => Exp::Hole(e.shift_in_range(range, by)),
        }
    }
}

impl Shift for Variable {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Variable { info, name, ctx: (), idx } = self;
        Variable { info: *info, name: name.clone(), ctx: (), idx: idx.shift_in_range(range, by) }
    }
}

impl Shift for TypCtor {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let TypCtor { info, name, args } = self;
        TypCtor { info: *info, name: name.clone(), args: args.shift_in_range(range, by) }
    }
}

impl Shift for Call {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Call { info, name, args } = self;
        Call { info: *info, name: name.clone(), args: args.shift_in_range(range, by) }
    }
}

impl Shift for DotCall {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let DotCall { info, exp, name, args } = self;
        DotCall {
            info: *info,
            exp: exp.shift_in_range(range.clone(), by),
            name: name.clone(),
            args: args.shift_in_range(range, by),
        }
    }
}

impl Shift for Anno {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Anno { info, exp, typ } = self;
        Anno {
            info: *info,
            exp: exp.shift_in_range(range.clone(), by),
            typ: typ.shift_in_range(range, by),
        }
    }
}

impl Shift for Type {
    fn shift_in_range<R: ShiftRange>(&self, _range: R, _by: (isize, isize)) -> Self {
        let Type { info } = self;
        Type { info: *info }
    }
}

impl Shift for LocalMatch {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let LocalMatch { info, ctx: (), name, on_exp, motive, ret_typ: (), body } = self;
        LocalMatch {
            info: *info,
            ctx: (),
            name: name.clone(),
            on_exp: on_exp.shift_in_range(range.clone(), by),
            motive: motive.shift_in_range(range.clone(), by),
            ret_typ: (),
            body: body.shift_in_range(range, by),
        }
    }
}

impl Shift for LocalComatch {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let LocalComatch { info, ctx: (), name, is_lambda_sugar, body } = self;
        LocalComatch {
            info: *info,
            ctx: (),
            name: name.clone(),
            is_lambda_sugar: *is_lambda_sugar,
            body: body.shift_in_range(range, by),
        }
    }
}

impl Shift for Hole {
    fn shift_in_range<R: ShiftRange>(&self, _range: R, _by: (isize, isize)) -> Self {
        let Hole { info } = self;
        Hole { info: *info }
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
