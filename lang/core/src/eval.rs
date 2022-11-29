use std::rc::Rc;

use syntax::ctx::{Bind, Context};
use syntax::env::*;
use syntax::ust::{self, Exp, Prg, Type};
use syntax::val::{self, Closure, Neu, Val};

pub trait Eval {
    type Val;

    fn eval(&self, prg: &Prg, env: &mut Env) -> Result<Self::Val, EvalError>;
}

pub trait Apply {
    fn apply(self, prg: &Prg, args: &[Rc<Val>]) -> Result<Rc<Val>, EvalError>;
}

#[derive(Debug)]
pub enum EvalError {}

impl Eval for Exp {
    type Val = Rc<Val>;

    fn eval(&self, prg: &Prg, env: &mut Env) -> Result<Self::Val, EvalError> {
        let res = match self {
            Exp::Var { idx, .. } => env.lookup(*idx),
            Exp::TypCtor { info, name, args } => Rc::new(Val::TypCtor {
                info: info.clone(),
                name: name.clone(),
                args: args.eval(prg, env)?,
            }),
            Exp::Ctor { info, name, args } => Rc::new(Val::Ctor {
                info: info.clone(),
                name: name.clone(),
                args: args.eval(prg, env)?,
            }),
            Exp::Dtor { info, exp, name, args } => {
                let exp = exp.eval(prg, env)?;
                let args = args.eval(prg, env)?;
                eval_dtor(prg, env, info, exp, name, args)?
            }
            Exp::Anno { exp, .. } => exp.eval(prg, env)?,
            Exp::Type { info } => Rc::new(Val::Type { info: info.clone() }),
            Exp::Match { name, on_exp, body, .. } => {
                eval_match(prg, name, on_exp.eval(prg, env)?, body.eval(prg, env)?)?
            }
            Exp::Comatch { info, name, body } => Rc::new(Val::Comatch {
                info: info.clone(),
                name: name.clone(),
                body: body.eval(prg, env)?,
            }),
        };
        Ok(res)
    }
}

fn eval_dtor(
    prg: &Prg,
    env: &mut Env,
    info: &ust::Info,
    exp: Rc<Val>,
    dtor_name: &str,
    dtor_args: Vec<Rc<Val>>,
) -> Result<Rc<Val>, EvalError> {
    match (*exp).clone() {
        Val::Ctor { name: ctor_name, args: ctor_args, .. } => {
            let type_decl = prg.decls.type_decl_for_member(&ctor_name);
            match type_decl {
                Type::Data(_) => {
                    let ust::Def { body, .. } = prg.decls.def(dtor_name);
                    let body = env.bind_iter(dtor_args.iter(), |env| body.eval(prg, env))?;
                    beta_match(prg, body, &ctor_name, &ctor_args)
                }
                Type::Codata(_) => {
                    let ust::Codef { body, .. } = prg.decls.codef(&ctor_name);
                    let body = env.bind_iter(ctor_args.iter(), |env| body.eval(prg, env))?;
                    beta_comatch(prg, body, dtor_name, &dtor_args)
                }
            }
        }
        Val::Comatch { body, .. } => beta_comatch(prg, body, dtor_name, &dtor_args),
        Val::Neu { exp } => Ok(Rc::new(Val::Neu {
            exp: Neu::Dtor {
                info: info.clone(),
                exp: Rc::new(exp),
                name: dtor_name.to_owned(),
                args: dtor_args,
            },
        })),
        _ => todo!(),
    }
}

fn eval_match(
    prg: &Prg,
    match_name: &str,
    on_exp: Rc<Val>,
    body: val::Match,
) -> Result<Rc<Val>, EvalError> {
    match (*on_exp).clone() {
        Val::Ctor { name: ctor_name, args, .. } => beta_match(prg, body, &ctor_name, &args),
        Val::Neu { exp } => Ok(Rc::new(Val::Neu {
            exp: Neu::Match {
                info: val::Info::empty(),
                name: match_name.to_owned(),
                on_exp: Rc::new(exp),
                body,
            },
        })),
        _ => todo!(),
    }
}

fn beta_match(
    prg: &Prg,
    body: val::Match,
    ctor_name: &str,
    args: &[Rc<Val>],
) -> Result<Rc<Val>, EvalError> {
    let case = body.cases.into_iter().find(|case| case.name == ctor_name).ok_or_else(|| todo!())?;
    let val::Case { body, .. } = case;
    let body = body.ok_or_else(|| todo!())?;
    body.apply(prg, args)
}

fn beta_comatch(
    prg: &Prg,
    body: val::Comatch,
    dtor_name: &str,
    args: &[Rc<Val>],
) -> Result<Rc<Val>, EvalError> {
    let cocase =
        body.cases.into_iter().find(|cocase| cocase.name == dtor_name).ok_or_else(|| todo!())?;
    let val::Cocase { body, .. } = cocase;
    let body = body.ok_or_else(|| todo!())?;
    body.apply(prg, args)
}

impl Eval for ust::Match {
    type Val = val::Match;

    fn eval(&self, prg: &Prg, env: &mut Env) -> Result<Self::Val, EvalError> {
        let ust::Match { info, cases } = self;

        Ok(val::Match { info: info.clone(), cases: cases.eval(prg, env)? })
    }
}

impl Eval for ust::Comatch {
    type Val = val::Comatch;

    fn eval(&self, prg: &Prg, env: &mut Env) -> Result<Self::Val, EvalError> {
        let ust::Comatch { info, cases } = self;

        Ok(val::Comatch { info: info.clone(), cases: cases.eval(prg, env)? })
    }
}

impl Eval for ust::Case {
    type Val = val::Case;

    fn eval(&self, _prg: &Prg, env: &mut Env) -> Result<Self::Val, EvalError> {
        let ust::Case { info, name, args, body } = self;

        let body = body.as_ref().map(|body| Closure { body: body.clone(), env: env.clone() });

        Ok(val::Case { info: info.clone(), name: name.clone(), args: args.clone(), body })
    }
}

impl Eval for ust::Cocase {
    type Val = val::Cocase;

    fn eval(&self, _prg: &Prg, env: &mut Env) -> Result<Self::Val, EvalError> {
        let ust::Cocase { info, name, args, body } = self;

        let body = body.as_ref().map(|body| Closure { body: body.clone(), env: env.clone() });

        Ok(val::Cocase { info: info.clone(), name: name.clone(), args: args.clone(), body })
    }
}

impl Apply for Closure {
    fn apply(mut self, prg: &Prg, args: &[Rc<Val>]) -> Result<Rc<Val>, EvalError> {
        self.env.bind_iter(args.iter(), |env| self.body.eval(prg, env))
    }
}

impl<T: Eval> Eval for Vec<T> {
    type Val = Vec<T::Val>;

    fn eval(&self, prg: &Prg, env: &mut Env) -> Result<Self::Val, EvalError> {
        self.iter().map(|x| x.eval(prg, env)).collect()
    }
}

impl Eval for Rc<Exp> {
    type Val = Rc<Val>;

    fn eval(&self, prg: &Prg, env: &mut Env) -> Result<Self::Val, EvalError> {
        (**self).eval(prg, env)
    }
}
