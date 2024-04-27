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
        let Variable { span, info, name, ctx: _, idx } = self;
        Variable {
            span: *span,
            info: *info,
            name: name.clone(),
            ctx: None,
            idx: idx.shift_in_range(range, by),
        }
    }
}

impl Shift for TypCtor {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let TypCtor { span, info, name, args } = self;
        TypCtor {
            span: *span,
            info: *info,
            name: name.clone(),
            args: args.shift_in_range(range, by),
        }
    }
}

impl Shift for Call {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Call { span, info, name, args } = self;
        Call { span: *span, info: *info, name: name.clone(), args: args.shift_in_range(range, by) }
    }
}

impl Shift for DotCall {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let DotCall { span, info, exp, name, args } = self;
        DotCall {
            span: *span,
            info: *info,
            exp: exp.shift_in_range(range.clone(), by),
            name: name.clone(),
            args: args.shift_in_range(range, by),
        }
    }
}

impl Shift for Anno {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Anno { span, info, exp, typ } = self;
        Anno {
            span: *span,
            info: *info,
            exp: exp.shift_in_range(range.clone(), by),
            typ: typ.shift_in_range(range, by),
        }
    }
}

impl Shift for Type {
    fn shift_in_range<R: ShiftRange>(&self, _range: R, _by: (isize, isize)) -> Self {
        let Type { span, info } = self;
        Type { span: *span, info: *info }
    }
}

impl Shift for LocalMatch {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let LocalMatch { span, info, ctx: _, name, on_exp, motive, ret_typ: _, body } = self;
        LocalMatch {
            span: *span,
            info: *info,
            ctx: None,
            name: name.clone(),
            on_exp: on_exp.shift_in_range(range.clone(), by),
            motive: motive.shift_in_range(range.clone(), by),
            ret_typ: None,
            body: body.shift_in_range(range, by),
        }
    }
}

impl Shift for LocalComatch {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let LocalComatch { span, info, ctx: _, name, is_lambda_sugar, body } = self;
        LocalComatch {
            span: *span,
            info: *info,
            ctx: None,
            name: name.clone(),
            is_lambda_sugar: *is_lambda_sugar,
            body: body.shift_in_range(range, by),
        }
    }
}

impl Shift for Hole {
    fn shift_in_range<R: ShiftRange>(&self, _range: R, _by: (isize, isize)) -> Self {
        let Hole { span, info } = self;
        Hole { span: *span, info: *info }
    }
}

impl Shift for Motive {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Motive { span, param, ret_typ } = self;

        Motive {
            span: *span,
            param: param.clone(),
            ret_typ: ret_typ.shift_in_range(range.shift(1), by),
        }
    }
}

impl Shift for Match {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Match { span, cases, omit_absurd } = self;
        Match { span: *span, cases: cases.shift_in_range(range, by), omit_absurd: *omit_absurd }
    }
}

impl Shift for Case {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let Case { span, name, params, body } = self;
        Case {
            span: *span,
            name: name.clone(),
            params: params.clone(),
            body: body.shift_in_range(range.shift(1), by),
        }
    }
}

impl Shift for TypApp {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        let TypApp { span, info, name, args } = self;
        TypApp {
            span: *span,
            info: *info,
            name: name.clone(),
            args: args.shift_in_range(range, by),
        }
    }
}

impl Shift for Args {
    fn shift_in_range<R: ShiftRange>(&self, range: R, by: (isize, isize)) -> Self {
        Self { args: self.args.shift_in_range(range, by) }
    }
}
