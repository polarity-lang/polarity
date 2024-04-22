use std::rc::Rc;

use codespan::Span;
use syntax::ctx::{BindContext, Context};
use syntax::ust;
use tracer::trace;

use crate::normalizer::env::*;
use crate::normalizer::val::{self, Closure, Neu, Val};

use crate::result::*;

pub trait Eval {
    type Val;

    fn eval(&self, prg: &ust::Prg, env: &mut Env) -> Result<Self::Val, TypeError>;
}

pub trait Apply {
    fn apply(self, prg: &ust::Prg, args: &[Rc<Val>]) -> Result<Rc<Val>, TypeError>;
}

impl Eval for ust::Exp {
    type Val = Rc<Val>;

    #[trace("{:P} |- {:P} ▷ {return:P}", env, self, std::convert::identity)]
    fn eval(&self, prg: &ust::Prg, env: &mut Env) -> Result<Self::Val, TypeError> {
        let res = match self {
            ust::Exp::Variable(ust::Variable { idx, .. }) => env.lookup(*idx),
            ust::Exp::TypCtor(ust::TypCtor { info, name, args }) => Rc::new(Val::TypCtor {
                info: *info,
                name: name.clone(),
                args: args.eval(prg, env)?,
            }),
            ust::Exp::Call(ust::Call { info, name, args }) => {
                Rc::new(Val::Ctor { info: *info, name: name.clone(), args: args.eval(prg, env)? })
            }
            ust::Exp::DotCall(ust::DotCall { info, exp, name, args }) => {
                let exp = exp.eval(prg, env)?;
                let args = args.eval(prg, env)?;
                eval_dtor(prg, info, exp, name, args)?
            }
            ust::Exp::Anno(ust::Anno { exp, .. }) => exp.eval(prg, env)?,
            ust::Exp::Type(ust::Type { info }) => Rc::new(Val::Type { info: *info }),
            ust::Exp::LocalMatch(ust::LocalMatch { name, on_exp, body, .. }) => {
                eval_match(prg, name, on_exp.eval(prg, env)?, body.eval(prg, env)?)?
            }
            ust::Exp::LocalComatch(ust::LocalComatch {
                info,
                ctx: (),
                name,
                is_lambda_sugar,
                body,
            }) => Rc::new(Val::Comatch {
                info: *info,
                name: name.clone(),
                is_lambda_sugar: *is_lambda_sugar,
                body: body.eval(prg, env)?,
            }),
            ust::Exp::Hole(ust::Hole { info }) => {
                Rc::new(Val::Neu { exp: Neu::Hole { info: *info } })
            }
        };
        Ok(res)
    }
}

fn eval_dtor(
    prg: &ust::Prg,
    info: &Option<Span>,
    exp: Rc<Val>,
    dtor_name: &str,
    dtor_args: Vec<Rc<Val>>,
) -> Result<Rc<Val>, TypeError> {
    match (*exp).clone() {
        Val::Ctor { name: ctor_name, args: ctor_args, info } => {
            let type_decl = prg.decls.type_decl_for_member(&ctor_name, info)?;
            match type_decl {
                ust::DataCodata::Data(_) => {
                    let ust::Def { body, .. } = prg.decls.def(dtor_name, None)?;
                    let body =
                        Env::empty().bind_iter(dtor_args.iter(), |env| body.eval(prg, env))?;
                    beta_match(prg, body, &ctor_name, &ctor_args)
                }
                ust::DataCodata::Codata(_) => {
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
    prg: &ust::Prg,
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
    prg: &ust::Prg,
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
    prg: &ust::Prg,
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

    fn eval(&self, prg: &ust::Prg, env: &mut Env) -> Result<Self::Val, TypeError> {
        let ust::Match { info, cases, omit_absurd } = self;

        Ok(val::Match { info: *info, cases: cases.eval(prg, env)?, omit_absurd: *omit_absurd })
    }
}

impl Eval for ust::Case {
    type Val = val::Case;

    fn eval(&self, _prg: &ust::Prg, env: &mut Env) -> Result<Self::Val, TypeError> {
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

    fn eval(&self, prg: &ust::Prg, env: &mut Env) -> Result<Self::Val, TypeError> {
        let ust::TypApp { info, name, args } = self;

        Ok(val::TypApp { info: *info, name: name.clone(), args: args.eval(prg, env)? })
    }
}

impl Eval for ust::Args {
    type Val = Vec<Rc<Val>>;

    fn eval(&self, prg: &ust::Prg, env: &mut Env) -> Result<Self::Val, TypeError> {
        self.args.eval(prg, env)
    }
}

impl Apply for Closure {
    fn apply(mut self, prg: &ust::Prg, args: &[Rc<Val>]) -> Result<Rc<Val>, TypeError> {
        self.env.bind_iter(args.iter(), |env| self.body.eval(prg, env))
    }
}

impl<T: Eval> Eval for Vec<T> {
    type Val = Vec<T::Val>;

    fn eval(&self, prg: &ust::Prg, env: &mut Env) -> Result<Self::Val, TypeError> {
        self.iter().map(|x| x.eval(prg, env)).collect()
    }
}

impl Eval for Rc<ust::Exp> {
    type Val = Rc<Val>;

    fn eval(&self, prg: &ust::Prg, env: &mut Env) -> Result<Self::Val, TypeError> {
        (**self).eval(prg, env)
    }
}
