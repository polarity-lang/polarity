use std::rc::Rc;

use super::val;
use syntax::common::*;
use syntax::ctx::BindContext;
use syntax::ust;
use tracer::trace;

use super::eval::Eval;

use crate::result::*;

pub trait ReadBack {
    type Nf;

    fn read_back(&self, prg: &ust::Prg) -> Result<Self::Nf, TypeError>;
}

impl ReadBack for val::Val {
    type Nf = ust::Exp;

    #[trace("↓{:P} ~> {return:P}", self, std::convert::identity)]
    fn read_back(&self, prg: &ust::Prg) -> Result<Self::Nf, TypeError> {
        let res = match self {
            val::Val::TypCtor { info, name, args } => ust::Exp::TypCtor(ust::TypCtor {
                info: *info,
                name: name.clone(),
                args: ust::Args { args: args.read_back(prg)? },
            }),
            val::Val::Ctor { info, name, args } => ust::Exp::Call(ust::Call {
                info: *info,
                name: name.clone(),
                args: ust::Args { args: args.read_back(prg)? },
            }),
            val::Val::Type { info } => ust::Exp::Type(ust::Type { info: *info }),
            val::Val::Comatch { info, name, is_lambda_sugar, body } => {
                ust::Exp::LocalComatch(ust::LocalComatch {
                    info: *info,
                    ctx: (),
                    name: name.clone(),
                    is_lambda_sugar: *is_lambda_sugar,
                    body: body.read_back(prg)?,
                })
            }
            val::Val::Neu { exp } => exp.read_back(prg)?,
        };
        Ok(res)
    }
}

impl ReadBack for val::Neu {
    type Nf = ust::Exp;

    fn read_back(&self, prg: &ust::Prg) -> Result<Self::Nf, TypeError> {
        let res = match self {
            val::Neu::Var { info, name, idx } => ust::Exp::Variable(ust::Variable {
                info: *info,
                ctx: (),
                name: name.clone(),
                idx: *idx,
            }),
            val::Neu::Dtor { info, exp, name, args } => ust::Exp::DotCall(ust::DotCall {
                info: *info,
                exp: exp.read_back(prg)?,
                name: name.clone(),
                args: ust::Args { args: args.read_back(prg)? },
            }),
            val::Neu::Match { info, name, on_exp, body } => ust::Exp::LocalMatch(ust::LocalMatch {
                info: *info,
                ctx: (),
                motive: None,
                ret_typ: (),
                name: name.clone(),
                on_exp: on_exp.read_back(prg)?,
                body: body.read_back(prg)?,
            }),
            val::Neu::Hole { info } => ust::Exp::Hole(ust::Hole { info: *info }),
        };
        Ok(res)
    }
}

impl ReadBack for val::Match {
    type Nf = ust::Match;

    fn read_back(&self, prg: &ust::Prg) -> Result<Self::Nf, TypeError> {
        let val::Match { info, cases, omit_absurd } = self;
        Ok(ust::Match { info: *info, cases: cases.read_back(prg)?, omit_absurd: *omit_absurd })
    }
}

impl ReadBack for val::Case {
    type Nf = ust::Case;

    fn read_back(&self, prg: &ust::Prg) -> Result<Self::Nf, TypeError> {
        let val::Case { info, name, args, body } = self;

        Ok(ust::Case {
            info: *info,
            name: name.clone(),
            args: args.clone(),
            body: body.read_back(prg)?,
        })
    }
}

impl ReadBack for val::TypApp {
    type Nf = ust::TypApp;

    fn read_back(&self, prg: &ust::Prg) -> Result<Self::Nf, TypeError> {
        let val::TypApp { info, name, args } = self;

        Ok(ust::TypApp {
            info: *info,
            name: name.clone(),
            args: ust::Args { args: args.read_back(prg)? },
        })
    }
}

impl ReadBack for val::Closure {
    type Nf = Rc<ust::Exp>;

    fn read_back(&self, prg: &ust::Prg) -> Result<Self::Nf, TypeError> {
        let args: Vec<Rc<val::Val>> = (0..self.n_args)
            .rev()
            .map(|snd| val::Val::Neu {
                exp: val::Neu::Var { info: None, name: "".to_owned(), idx: Idx { fst: 0, snd } },
            })
            .map(Rc::new)
            .collect();
        self.env
            .shift((1, 0))
            .bind_iter(args.iter(), |env| self.body.eval(prg, env))?
            .read_back(prg)
    }
}

impl<T: ReadBack> ReadBack for Vec<T> {
    type Nf = Vec<T::Nf>;

    fn read_back(&self, prg: &ust::Prg) -> Result<Self::Nf, TypeError> {
        self.iter().map(|x| x.read_back(prg)).collect()
    }
}

impl<T: ReadBack> ReadBack for Rc<T> {
    type Nf = Rc<T::Nf>;

    fn read_back(&self, prg: &ust::Prg) -> Result<Self::Nf, TypeError> {
        (**self).read_back(prg).map(Rc::new)
    }
}

impl<T: ReadBack> ReadBack for Option<T> {
    type Nf = Option<T::Nf>;

    fn read_back(&self, prg: &ust::Prg) -> Result<Self::Nf, TypeError> {
        self.as_ref().map(|x| x.read_back(prg)).transpose()
    }
}
