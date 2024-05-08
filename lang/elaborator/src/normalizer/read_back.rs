use std::rc::Rc;

use super::val;
use syntax::ast::*;
use syntax::common::*;
use syntax::ctx::BindContext;
use tracer::trace;

use super::eval::Eval;

use crate::result::*;

pub trait ReadBack {
    type Nf;

    fn read_back(&self, prg: &Module) -> Result<Self::Nf, TypeError>;
}

impl ReadBack for val::TypCtor {
    type Nf = TypCtor;

    fn read_back(&self, prg: &Module) -> Result<Self::Nf, TypeError> {
        let val::TypCtor { span, name, args } = self;
        Ok(TypCtor { span: *span, name: name.clone(), args: Args { args: args.read_back(prg)? } })
    }
}

impl ReadBack for val::Call {
    type Nf = Call;

    fn read_back(&self, prg: &Module) -> Result<Self::Nf, TypeError> {
        let val::Call { span, kind, name, args } = self;
        Ok(Call {
            span: *span,
            kind: *kind,
            name: name.clone(),
            args: Args { args: args.read_back(prg)? },
            inferred_type: None,
        })
    }
}

impl ReadBack for val::TypeUniv {
    type Nf = TypeUniv;

    fn read_back(&self, _prg: &Module) -> Result<Self::Nf, TypeError> {
        let val::TypeUniv { span } = self;
        Ok(TypeUniv { span: *span })
    }
}

impl ReadBack for val::LocalComatch {
    type Nf = LocalComatch;
    fn read_back(&self, prg: &Module) -> Result<Self::Nf, TypeError> {
        let val::LocalComatch { span, name, is_lambda_sugar, body } = self;
        Ok(LocalComatch {
            span: *span,
            ctx: None,
            name: name.clone(),
            is_lambda_sugar: *is_lambda_sugar,
            body: body.read_back(prg)?,
            inferred_type: None,
        })
    }
}
impl ReadBack for val::Val {
    type Nf = Exp;

    #[trace("↓{:P} ~> {return:P}", self, std::convert::identity)]
    fn read_back(&self, prg: &Module) -> Result<Self::Nf, TypeError> {
        let res = match self {
            val::Val::TypCtor(e) => e.read_back(prg)?.into(),
            val::Val::Call(e) => e.read_back(prg)?.into(),
            val::Val::TypeUniv(e) => e.read_back(prg)?.into(),
            val::Val::LocalComatch(e) => e.read_back(prg)?.into(),
            val::Val::Neu(exp) => exp.read_back(prg)?,
        };
        Ok(res)
    }
}

impl ReadBack for val::Variable {
    type Nf = Variable;

    fn read_back(&self, _prg: &Module) -> Result<Self::Nf, TypeError> {
        let val::Variable { span, name, idx } = self;
        Ok(Variable { span: *span, idx: *idx, name: name.clone(), inferred_type: None })
    }
}

impl ReadBack for val::DotCall {
    type Nf = DotCall;

    fn read_back(&self, prg: &Module) -> Result<Self::Nf, TypeError> {
        let val::DotCall { span, kind, exp, name, args } = self;
        Ok(DotCall {
            span: *span,
            kind: *kind,
            exp: exp.read_back(prg)?,
            name: name.clone(),
            args: Args { args: args.read_back(prg)? },
            inferred_type: None,
        })
    }
}

impl ReadBack for val::LocalMatch {
    type Nf = LocalMatch;

    fn read_back(&self, prg: &Module) -> Result<Self::Nf, TypeError> {
        let val::LocalMatch { span, name, on_exp, body } = self;
        Ok(LocalMatch {
            span: *span,
            ctx: None,
            motive: None,
            ret_typ: None,
            name: name.clone(),
            on_exp: on_exp.read_back(prg)?,
            body: body.read_back(prg)?,
            inferred_type: None,
        })
    }
}

impl ReadBack for val::Hole {
    type Nf = Hole;

    fn read_back(&self, _prg: &Module) -> Result<Self::Nf, TypeError> {
        let val::Hole { span } = self;
        Ok(Hole { span: *span, inferred_type: None, inferred_ctx: None })
    }
}

impl ReadBack for val::Neu {
    type Nf = Exp;

    fn read_back(&self, prg: &Module) -> Result<Self::Nf, TypeError> {
        let res = match self {
            val::Neu::Variable(e) => e.read_back(prg)?.into(),
            val::Neu::DotCall(e) => e.read_back(prg)?.into(),
            val::Neu::LocalMatch(e) => e.read_back(prg)?.into(),
            val::Neu::Hole(e) => e.read_back(prg)?.into(),
        };
        Ok(res)
    }
}

impl ReadBack for val::Match {
    type Nf = Match;

    fn read_back(&self, prg: &Module) -> Result<Self::Nf, TypeError> {
        let val::Match { span, cases, omit_absurd } = self;
        Ok(Match { span: *span, cases: cases.read_back(prg)?, omit_absurd: *omit_absurd })
    }
}

impl ReadBack for val::Case {
    type Nf = Case;

    fn read_back(&self, prg: &Module) -> Result<Self::Nf, TypeError> {
        let val::Case { span, name, params, body } = self;

        Ok(Case {
            span: *span,
            name: name.clone(),
            params: params.clone(),
            body: body.read_back(prg)?,
        })
    }
}

impl ReadBack for val::Closure {
    type Nf = Rc<Exp>;

    fn read_back(&self, prg: &Module) -> Result<Self::Nf, TypeError> {
        let args: Vec<Rc<val::Val>> = (0..self.n_args)
            .rev()
            .map(|snd| {
                val::Val::Neu(val::Neu::Variable(val::Variable {
                    span: None,
                    name: "".to_owned(),
                    idx: Idx { fst: 0, snd },
                }))
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

    fn read_back(&self, prg: &Module) -> Result<Self::Nf, TypeError> {
        self.iter().map(|x| x.read_back(prg)).collect()
    }
}

impl<T: ReadBack> ReadBack for Rc<T> {
    type Nf = Rc<T::Nf>;

    fn read_back(&self, prg: &Module) -> Result<Self::Nf, TypeError> {
        (**self).read_back(prg).map(Rc::new)
    }
}

impl<T: ReadBack> ReadBack for Option<T> {
    type Nf = Option<T::Nf>;

    fn read_back(&self, prg: &Module) -> Result<Self::Nf, TypeError> {
        self.as_ref().map(|x| x.read_back(prg)).transpose()
    }
}
