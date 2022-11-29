use crate::common::*;

use super::def::*;

impl ShiftCutoff for Val {
    fn shift_cutoff(&self, cutoff: usize, by: (isize, isize)) -> Self {
        match self {
            Val::TypCtor { info, name, args } => Val::TypCtor {
                info: info.clone(),
                name: name.clone(),
                args: args.shift_cutoff(cutoff, by),
            },
            Val::Ctor { info, name, args } => Val::Ctor {
                info: info.clone(),
                name: name.clone(),
                args: args.shift_cutoff(cutoff, by),
            },
            Val::Type { info } => Val::Type { info: info.clone() },
            Val::Comatch { info, name, body } => Val::Comatch {
                info: info.clone(),
                name: name.clone(),
                body: body.shift_cutoff(cutoff, by),
            },
            Val::Neu { exp } => Val::Neu { exp: exp.shift_cutoff(cutoff, by) },
        }
    }
}

impl ShiftCutoff for Neu {
    fn shift_cutoff(&self, cutoff: usize, by: (isize, isize)) -> Self {
        match self {
            Neu::Var { info, name, idx } => Neu::Var {
                info: info.clone(),
                name: name.clone(),
                idx: idx.shift_cutoff(cutoff, by),
            },
            Neu::Dtor { info, exp, name, args } => Neu::Dtor {
                info: info.clone(),
                exp: exp.shift_cutoff(cutoff, by),
                name: name.clone(),
                args: args.shift_cutoff(cutoff, by),
            },
            Neu::Match { info, name, on_exp, body } => Neu::Match {
                info: info.clone(),
                name: name.clone(),
                on_exp: on_exp.shift_cutoff(cutoff, by),
                body: body.shift_cutoff(cutoff, by),
            },
        }
    }
}

impl ShiftCutoff for Match {
    fn shift_cutoff(&self, cutoff: usize, by: (isize, isize)) -> Self {
        let Match { info, cases } = self;
        Match { info: info.clone(), cases: cases.shift_cutoff(cutoff, by) }
    }
}

impl ShiftCutoff for Comatch {
    fn shift_cutoff(&self, cutoff: usize, by: (isize, isize)) -> Self {
        let Comatch { info, cases } = self;
        Comatch { info: info.clone(), cases: cases.shift_cutoff(cutoff, by) }
    }
}

impl ShiftCutoff for Case {
    fn shift_cutoff(&self, cutoff: usize, by: (isize, isize)) -> Self {
        let Case { info, name, args, body } = self;

        Case {
            info: info.clone(),
            name: name.clone(),
            args: args.clone(),
            body: body.shift_cutoff(cutoff + 1, by),
        }
    }
}

impl ShiftCutoff for Cocase {
    fn shift_cutoff(&self, cutoff: usize, by: (isize, isize)) -> Self {
        let Cocase { info, name, args, body } = self;

        Cocase {
            info: info.clone(),
            name: name.clone(),
            args: args.clone(),
            body: body.shift_cutoff(cutoff + 1, by),
        }
    }
}

impl ShiftCutoff for Closure {
    fn shift_cutoff(&self, cutoff: usize, by: (isize, isize)) -> Self {
        let Closure { env, body } = self;

        Closure { env: env.shift_cutoff(cutoff, by), body: body.clone() }
    }
}
