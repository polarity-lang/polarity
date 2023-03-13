use std::rc::Rc;

use syntax::common::*;
use syntax::ctx::Bind;
use syntax::nf;
use syntax::ust::Prg;
use syntax::val;
use tracer::trace;

use super::eval::Eval;

use super::result::*;

pub trait ReadBack {
    type Nf;

    fn read_back(&self, prg: &Prg) -> Result<Self::Nf, EvalError>;
}

impl ReadBack for val::Val {
    type Nf = nf::Nf;

    #[trace("↓{:P} ~> {return:P}", self, data::id)]
    fn read_back(&self, prg: &Prg) -> Result<Self::Nf, EvalError> {
        let res = match self {
            val::Val::TypCtor { info, name, args } => nf::Nf::TypCtor {
                info: info.clone(),
                name: name.clone(),
                args: args.read_back(prg)?,
            },
            val::Val::Ctor { info, name, args } => {
                nf::Nf::Ctor { info: info.clone(), name: name.clone(), args: args.read_back(prg)? }
            }
            val::Val::Type { info } => nf::Nf::Type { info: info.clone() },
            val::Val::Comatch { info, name, body } => nf::Nf::Comatch {
                info: info.clone(),
                name: name.clone(),
                body: body.read_back(prg)?,
            },
            val::Val::Neu { exp } => nf::Nf::Neu { exp: exp.read_back(prg)? },
        };
        Ok(res)
    }
}

impl ReadBack for val::Neu {
    type Nf = nf::Neu;

    fn read_back(&self, prg: &Prg) -> Result<Self::Nf, EvalError> {
        let res = match self {
            val::Neu::Var { info, name, idx } => {
                nf::Neu::Var { info: info.clone(), name: name.clone(), idx: *idx }
            }
            val::Neu::Dtor { info, exp, name, args } => nf::Neu::Dtor {
                info: info.clone(),
                exp: exp.read_back(prg)?,
                name: name.clone(),
                args: args.read_back(prg)?,
            },
            val::Neu::Match { info, name, on_exp, body } => nf::Neu::Match {
                info: info.clone(),
                name: name.clone(),
                on_exp: on_exp.read_back(prg)?,
                body: body.read_back(prg)?,
            },
            val::Neu::Hole { info, kind } => nf::Neu::Hole { info: info.clone(), kind: *kind },
        };
        Ok(res)
    }
}

impl ReadBack for val::Match {
    type Nf = nf::Match;

    fn read_back(&self, prg: &Prg) -> Result<Self::Nf, EvalError> {
        let val::Match { info, cases } = self;
        Ok(nf::Match { info: info.clone(), cases: cases.read_back(prg)? })
    }
}

impl ReadBack for val::Comatch {
    type Nf = nf::Comatch;

    fn read_back(&self, prg: &Prg) -> Result<Self::Nf, EvalError> {
        let val::Comatch { info, cases } = self;
        Ok(nf::Comatch { info: info.clone(), cases: cases.read_back(prg)? })
    }
}

impl ReadBack for val::Case {
    type Nf = nf::Case;

    fn read_back(&self, prg: &Prg) -> Result<Self::Nf, EvalError> {
        let val::Case { info, name, args, body } = self;

        Ok(nf::Case {
            info: info.clone(),
            name: name.clone(),
            args: args.clone(),
            body: body.read_back(prg)?,
        })
    }
}

impl ReadBack for val::Cocase {
    type Nf = nf::Cocase;

    fn read_back(&self, prg: &Prg) -> Result<Self::Nf, EvalError> {
        let val::Cocase { info, name, args, body } = self;

        Ok(nf::Cocase {
            info: info.clone(),
            name: name.clone(),
            args: args.clone(),
            body: body.read_back(prg)?,
        })
    }
}

impl ReadBack for val::TypApp {
    type Nf = nf::TypApp;

    fn read_back(&self, prg: &Prg) -> Result<Self::Nf, EvalError> {
        let val::TypApp { info, name, args } = self;

        Ok(nf::TypApp { info: info.clone(), name: name.clone(), args: args.read_back(prg)? })
    }
}

impl ReadBack for val::Closure {
    type Nf = Rc<nf::Nf>;

    fn read_back(&self, prg: &Prg) -> Result<Self::Nf, EvalError> {
        let args: Vec<Rc<val::Val>> = (0..self.n_args)
            .rev()
            .map(|snd| val::Val::Neu {
                exp: val::Neu::Var {
                    info: val::Info::empty(),
                    name: "".to_owned(),
                    idx: Idx { fst: 0, snd },
                },
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

    fn read_back(&self, prg: &Prg) -> Result<Self::Nf, EvalError> {
        self.iter().map(|x| x.read_back(prg)).collect()
    }
}

impl<T: ReadBack> ReadBack for Rc<T> {
    type Nf = Rc<T::Nf>;

    fn read_back(&self, prg: &Prg) -> Result<Self::Nf, EvalError> {
        (**self).read_back(prg).map(Rc::new)
    }
}

impl<T: ReadBack> ReadBack for Option<T> {
    type Nf = Option<T::Nf>;

    fn read_back(&self, prg: &Prg) -> Result<Self::Nf, EvalError> {
        self.as_ref().map(|x| x.read_back(prg)).transpose()
    }
}
