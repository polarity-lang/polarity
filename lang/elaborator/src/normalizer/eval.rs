use std::rc::Rc;

use codespan::Span;
use syntax::ctx::{BindContext, Context};
use syntax::ust::{self, Exp, Prg, Type};
use tracer::trace;

use crate::normalizer::env::*;
use crate::normalizer::val::{self, Closure, Neu, Val};

use crate::result::*;

pub trait Eval {
    type Val;

    fn eval(&self, prg: &Prg, env: &mut Env) -> Result<Self::Val, TypeError>;
}

pub trait Apply {
    fn apply(self, prg: &Prg, args: &[Rc<Val>]) -> Result<Rc<Val>, TypeError>;
}

impl Eval for Exp {
    type Val = Rc<Val>;

    #[trace("{:P} |- {:P} ▷ {return:P}", env, self, std::convert::identity)]
    fn eval(&self, prg: &Prg, env: &mut Env) -> Result<Self::Val, TypeError> {
        let res = match self {
            Exp::Var { idx, .. } => env.lookup(*idx),
            Exp::TypCtor { info, name, args } => Rc::new(Val::TypCtor {
                info: *info,
                name: name.clone(),
                args: args.eval(prg, env)?,
            }),
            Exp::Ctor { info, name, args } => {
                Rc::new(Val::Ctor { info: *info, name: name.clone(), args: args.eval(prg, env)? })
            }
            Exp::Dtor { info, exp, name, args } => {
                let exp = exp.eval(prg, env)?;
                let args = args.eval(prg, env)?;
                eval_dtor(prg, info, exp, name, args)?
            }
            Exp::Anno { exp, .. } => exp.eval(prg, env)?,
            Exp::Type { info } => Rc::new(Val::Type { info: *info }),
            Exp::Match { name, on_exp, body, .. } => {
                eval_match(prg, name, on_exp.eval(prg, env)?, body.eval(prg, env)?)?
            }
            Exp::Comatch { info, ctx: (), name, is_lambda_sugar, body } => Rc::new(Val::Comatch {
                info: *info,
                name: name.clone(),
                is_lambda_sugar: *is_lambda_sugar,
                body: body.eval(prg, env)?,
            }),
            Exp::Hole { info } => Rc::new(Val::Neu { exp: Neu::Hole { info: *info } }),
        };
        Ok(res)
    }
}

fn eval_dtor(
    prg: &Prg,
    info: &Option<Span>,
    exp: Rc<Val>,
    dtor_name: &str,
    dtor_args: Vec<Rc<Val>>,
) -> Result<Rc<Val>, TypeError> {
    match (*exp).clone() {
        Val::Ctor { name: ctor_name, args: ctor_args, info } => {
            let type_decl = prg.decls.type_decl_for_member(&ctor_name, info)?;
            match type_decl {
                Type::Data(_) => {
                    let ust::Def { body, .. } = prg.decls.def(dtor_name, None)?;
                    let body =
                        Env::empty().bind_iter(dtor_args.iter(), |env| body.eval(prg, env))?;
                    beta_match(prg, body, &ctor_name, &ctor_args)
                }
                Type::Codata(_) => {
                    let ust::Codef { body, .. } = prg.decls.codef(&ctor_name, None)?;
                    let body =
                        Env::empty().bind_iter(ctor_args.iter(), |env| body.eval(prg, env))?;
                    beta_comatch(prg, body, dtor_name, &dtor_args)
                }
            }
        }
        Val::Comatch { body, .. } => beta_comatch(prg, body, dtor_name, &dtor_args),
        Val::Neu { exp } => Ok(Rc::new(Val::Neu {
            exp: Neu::Dtor {
                info: *info,
                exp: Rc::new(exp),
                name: dtor_name.to_owned(),
                args: dtor_args,
            },
        })),
        _ => unreachable!(),
    }
}

fn eval_match(
    prg: &Prg,
    match_name: &ust::Label,
    on_exp: Rc<Val>,
    body: val::Match,
) -> Result<Rc<Val>, TypeError> {
    match (*on_exp).clone() {
        Val::Ctor { name: ctor_name, args, .. } => beta_match(prg, body, &ctor_name, &args),
        Val::Neu { exp } => Ok(Rc::new(Val::Neu {
            exp: Neu::Match { info: None, name: match_name.to_owned(), on_exp: Rc::new(exp), body },
        })),
        _ => unreachable!(),
    }
}

#[trace("{}(...).match {:P} ▷β {return:P}", ctor_name, body, std::convert::identity)]
fn beta_match(
    prg: &Prg,
    body: val::Match,
    ctor_name: &str,
    args: &[Rc<Val>],
) -> Result<Rc<Val>, TypeError> {
    let case = body.clone().cases.into_iter().find(|case| case.name == ctor_name).unwrap();
    let val::Case { body, .. } = case;
    let body = body.unwrap();
    body.apply(prg, args)
}

#[trace("comatch {:P}.{}(...) ▷β {return:P}", body, dtor_name, std::convert::identity)]
fn beta_comatch(
    prg: &Prg,
    body: val::Match,
    dtor_name: &str,
    args: &[Rc<Val>],
) -> Result<Rc<Val>, TypeError> {
    let cocase = body.clone().cases.into_iter().find(|cocase| cocase.name == dtor_name).unwrap();
    let val::Case { body, .. } = cocase;
    let body = body.unwrap();
    body.apply(prg, args)
}

impl Eval for ust::Match {
    type Val = val::Match;

    fn eval(&self, prg: &Prg, env: &mut Env) -> Result<Self::Val, TypeError> {
        let ust::Match { info, cases, omit_absurd } = self;

        Ok(val::Match { info: *info, cases: cases.eval(prg, env)?, omit_absurd: *omit_absurd })
    }
}

impl Eval for ust::Case {
    type Val = val::Case;

    fn eval(&self, _prg: &Prg, env: &mut Env) -> Result<Self::Val, TypeError> {
        let ust::Case { info, name, args, body } = self;

        let body = body.as_ref().map(|body| Closure {
            body: body.clone(),
            n_args: args.len(),
            env: env.clone(),
        });

        Ok(val::Case { info: *info, name: name.clone(), args: args.clone(), body })
    }
}

impl Eval for ust::TypApp {
    type Val = val::TypApp;

    fn eval(&self, prg: &Prg, env: &mut Env) -> Result<Self::Val, TypeError> {
        let ust::TypApp { info, name, args } = self;

        Ok(val::TypApp { info: *info, name: name.clone(), args: args.eval(prg, env)? })
    }
}

impl Eval for ust::Args {
    type Val = Vec<Rc<Val>>;

    fn eval(&self, prg: &Prg, env: &mut Env) -> Result<Self::Val, TypeError> {
        self.args.eval(prg, env)
    }
}

impl Apply for Closure {
    fn apply(mut self, prg: &Prg, args: &[Rc<Val>]) -> Result<Rc<Val>, TypeError> {
        self.env.bind_iter(args.iter(), |env| self.body.eval(prg, env))
    }
}

impl<T: Eval> Eval for Vec<T> {
    type Val = Vec<T::Val>;

    fn eval(&self, prg: &Prg, env: &mut Env) -> Result<Self::Val, TypeError> {
        self.iter().map(|x| x.eval(prg, env)).collect()
    }
}

impl Eval for Rc<Exp> {
    type Val = Rc<Val>;

    fn eval(&self, prg: &Prg, env: &mut Env) -> Result<Self::Val, TypeError> {
        (**self).eval(prg, env)
    }
}
