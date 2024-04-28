use std::rc::Rc;

use super::val;
use syntax::common::*;
use syntax::ctx::BindContext;
use syntax::generic::{TypeUniv, Variable};
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
            val::Val::TypCtor { span, name, args } => ust::Exp::TypCtor(ust::TypCtor {
                span: *span,
                info: (),
                name: name.clone(),
                args: ust::Args { args: args.read_back(prg)? },
            }),
            val::Val::Ctor { span, name, args } => ust::Exp::Call(ust::Call {
                span: *span,
                info: (),
                name: name.clone(),
                args: ust::Args { args: args.read_back(prg)? },
            }),
            val::Val::TypeUniv { span } => ust::Exp::TypeUniv(TypeUniv { span: *span }),
            val::Val::Comatch { span, name, is_lambda_sugar, body } => {
                ust::Exp::LocalComatch(ust::LocalComatch {
                    span: *span,
                    info: (),
                    ctx: None,
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
            val::Neu::Var { span, name, idx } => ust::Exp::Variable(Variable {
                span: *span,
                idx: *idx,
                name: name.clone(),
                inferred_type: None,
            }),
            val::Neu::Dtor { span, exp, name, args } => ust::Exp::DotCall(ust::DotCall {
                span: *span,
                info: (),
                exp: exp.read_back(prg)?,
                name: name.clone(),
                args: ust::Args { args: args.read_back(prg)? },
            }),
            val::Neu::Match { span, name, on_exp, body } => ust::Exp::LocalMatch(ust::LocalMatch {
                span: *span,
                info: (),
                ctx: None,
                motive: None,
                ret_typ: None,
                name: name.clone(),
                on_exp: on_exp.read_back(prg)?,
                body: body.read_back(prg)?,
            }),
            val::Neu::Hole { span } => ust::Exp::Hole(ust::Hole { span: *span, info: () }),
        };
        Ok(res)
    }
}

impl ReadBack for val::Match {
    type Nf = ust::Match;

    fn read_back(&self, prg: &ust::Prg) -> Result<Self::Nf, TypeError> {
        let val::Match { span, cases, omit_absurd } = self;
        Ok(ust::Match { span: *span, cases: cases.read_back(prg)?, omit_absurd: *omit_absurd })
    }
}

impl ReadBack for val::Case {
    type Nf = ust::Case;

    fn read_back(&self, prg: &ust::Prg) -> Result<Self::Nf, TypeError> {
        let val::Case { span, name, params, body } = self;

        Ok(ust::Case {
            span: *span,
            name: name.clone(),
            params: params.clone(),
            body: body.read_back(prg)?,
        })
    }
}

impl ReadBack for val::TypApp {
    type Nf = ust::TypCtor;

    fn read_back(&self, prg: &ust::Prg) -> Result<Self::Nf, TypeError> {
        let val::TypApp { span, name, args } = self;

        Ok(ust::TypCtor {
            span: *span,
            info: (),
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
                exp: val::Neu::Var { span: None, name: "".to_owned(), idx: Idx { fst: 0, snd } },
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
