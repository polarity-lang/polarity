use std::rc::Rc;

use log::trace;

use printer::types::Print;
use syntax::ast::*;
use syntax::ctx::{BindContext, Context};

use crate::normalizer::env::*;
use crate::normalizer::val::{self, Closure, Val};

use crate::result::*;

pub trait Eval {
    type Val;

    fn eval(&self, prg: &Module, env: &mut Env) -> Result<Self::Val, TypeError>;
}

pub trait Apply {
    fn apply(self, prg: &Module, args: &[Rc<Val>]) -> Result<Rc<Val>, TypeError>;
}

impl Eval for Exp {
    type Val = Rc<Val>;

    fn eval(&self, prg: &Module, env: &mut Env) -> Result<Self::Val, TypeError> {
        let e = match self {
            Exp::Variable(e) => e.eval(prg, env),
            Exp::TypCtor(e) => e.eval(prg, env),
            Exp::Call(e) => e.eval(prg, env),
            Exp::DotCall(e) => e.eval(prg, env),
            Exp::Anno(e) => e.eval(prg, env),
            Exp::TypeUniv(e) => e.eval(prg, env),
            Exp::LocalMatch(e) => e.eval(prg, env),
            Exp::LocalComatch(e) => e.eval(prg, env),
            Exp::Hole(e) => e.eval(prg, env),
        };
        trace!(
            "{} |- {} ▷ {}",
            env.print_to_colored_string(None),
            self.print_to_colored_string(None),
            e.print_to_colored_string(None)
        );
        e
    }
}

impl Eval for Variable {
    type Val = Rc<Val>;

    fn eval(&self, _prg: &Module, env: &mut Env) -> Result<Self::Val, TypeError> {
        let Variable { idx, .. } = self;
        Ok(env.lookup(*idx))
    }
}

impl Eval for TypCtor {
    type Val = Rc<Val>;

    fn eval(&self, prg: &Module, env: &mut Env) -> Result<Self::Val, TypeError> {
        let TypCtor { span, name, args } = self;
        Ok(Rc::new(
            val::TypCtor { span: *span, name: name.clone(), args: args.eval(prg, env)? }.into(),
        ))
    }
}

impl Eval for Call {
    type Val = Rc<Val>;

    fn eval(&self, prg: &Module, env: &mut Env) -> Result<Self::Val, TypeError> {
        let Call { span, name, kind, args, .. } = self;
        match kind {
            CallKind::LetBound => {
                let Let { attr, body, .. } = prg.top_level_let(name, *span)?;
                // We now have to distinguish two cases:
                // If the let-bound definition is transparent, then we substitute the
                // arguments for the body of the definition. If it is opaque, then
                // the further computation is blocked so we return a neutral value.
                if attr.attrs.contains(&Attribute::Transparent) {
                    let args = args.eval(prg, env)?;
                    return env.bind_iter(args.iter(), |env| body.eval(prg, env));
                } else {
                    Ok(Rc::new(Val::Neu(
                        val::OpaqueCall {
                            span: *span,
                            name: name.clone(),
                            args: args.eval(prg, env)?,
                        }
                        .into(),
                    )))
                }
            }
            CallKind::Constructor | CallKind::Codefinition => Ok(Rc::new(
                val::Call {
                    span: *span,
                    kind: *kind,
                    name: name.clone(),
                    args: args.eval(prg, env)?,
                }
                .into(),
            )),
        }
    }
}

impl Eval for DotCall {
    type Val = Rc<Val>;

    /// Evaluate a DotCall:
    ///
    /// ```text
    /// e.d(e_1,...)
    /// ┳ ┳ ━━━┳━━━
    /// ┃ ┃    ┗━━━━━━━ args
    /// ┃ ┗━━━━━━━━━━━━ name
    /// ┗━━━━━━━━━━━━━━ exp
    /// ```
    fn eval(&self, prg: &Module, env: &mut Env) -> Result<Self::Val, TypeError> {
        let DotCall { span, kind, exp, name, args, .. } = self;

        // We first evaluate `exp` and then the arguments `args` to `d` from left to right.
        let exp = exp.eval(prg, env)?;
        let args = args.eval(prg, env)?;

        match (*exp).clone() {
            Val::Call(val::Call { name: call_name, kind, args: call_args, .. }) => {
                match kind {
                    CallKind::Constructor => {
                        // The specific instance of the DotCall we are evaluating is:
                        //
                        // ```text
                        //  C(t_1,..).d(e_1,...)
                        //  ┳ ━━┳━━━  ┳ ━━━┳━━━
                        //  ┃   ┃     ┃    ┗━━━━ args
                        //  ┃   ┃     ┗━━━━━━━━━ name
                        //  ┃   ┗━━━━━━━━━━━━━━━ call_args
                        //  ┗━━━━━━━━━━━━━━━━━━━ call_name
                        // ```
                        //
                        // where `C` is the name of a constructor declared in a
                        // data type, and `d` is the name of a toplevel definition.

                        // First, we have to find the corresponding case in the toplevel definition `d`.
                        let Def { cases, .. } = prg.def(name, None)?;
                        let cases =
                            Env::empty().bind_iter(args.iter(), |env| cases.eval(prg, env))?;
                        let val::Case { body, .. } =
                            cases.clone().into_iter().find(|case| case.name == call_name).unwrap();

                        // Then we apply the body to the `call_args`.
                        body.clone().unwrap().apply(prg, &call_args)
                    }
                    CallKind::Codefinition => {
                        // The specific instance of the DotCall we are evaluating is:
                        //
                        // ```text
                        //  C(t_1,..).d(e_1,...)
                        //  ┳ ━━┳━━━  ┳ ━━━┳━━━
                        //  ┃   ┃     ┃    ┗━━━━ args
                        //  ┃   ┃     ┗━━━━━━━━━ name
                        //  ┃   ┗━━━━━━━━━━━━━━━ call_args
                        //  ┗━━━━━━━━━━━━━━━━━━━ call_name
                        // ```
                        //
                        // where `d` is the name of a destructor declared in a
                        // data type, and `C` is the name of a toplevel codefinition.

                        // First, we have to find the corresponding cocase in the toplevel
                        // codefinition `C`.
                        let Codef { cases, .. } = prg.codef(&call_name, None)?;
                        let cases =
                            Env::empty().bind_iter(call_args.iter(), |env| cases.eval(prg, env))?;
                        let val::Case { body, .. } =
                            cases.clone().into_iter().find(|cocase| cocase.name == *name).unwrap();

                        // Then we apply the body to the `args`.
                        body.clone().unwrap().apply(prg, &args)
                    }
                    CallKind::LetBound => {
                        // This case is unreachable because all let-bound calls have either already
                        // been replaced by their body (if they are transparent), or they have been
                        // turned into a neutral `OpaqueCall` if they are opaque.
                        unreachable!()
                    }
                }
            }
            Val::LocalComatch(val::LocalComatch { cases, .. }) => {
                // The specific instance of the DotCall we are evaluating is:
                //
                // ```text
                //  comatch { ... }.d(e_1,...)
                //            ━┳━   ┳ ━━━┳━━━
                //             ┃    ┃    ┗━━━━ args
                //             ┃    ┗━━━━━━━━━ name
                //             ┗━━━━━━━━━━━━━━ cases
                // ```
                //
                // where `d` is the name of a destructor declared in a
                // codata type.

                // First, we have to select the correct case from the comatch.
                let val::Case { body, .. } =
                    cases.clone().into_iter().find(|cocase| cocase.name == *name).unwrap();

                // Then we apply the body to the `args`.
                body.clone().unwrap().apply(prg, &args)
            }
            Val::Neu(exp) => {
                // The specific instance of the DotCall we are evaluating is:
                //
                // ```text
                // n.d(e_1,...)
                // ┳ ┳ ━━━┳━━━
                // ┃ ┃    ┗━━━━━━━ args
                // ┃ ┗━━━━━━━━━━━━ name
                // ┗━━━━━━━━━━━━━━ exp (Neutral value)
                // ```
                // Evaluation is blocked by the neutral value `n`.
                Ok(Rc::new(Val::Neu(
                    val::DotCall {
                        span: *span,
                        kind: *kind,
                        exp: Rc::new(exp),
                        name: name.to_owned(),
                        args,
                    }
                    .into(),
                )))
            }
            _ => unreachable!(),
        }
    }
}

impl Eval for Anno {
    type Val = Rc<Val>;

    fn eval(&self, prg: &Module, env: &mut Env) -> Result<Self::Val, TypeError> {
        let Anno { exp, .. } = self;
        exp.eval(prg, env)
    }
}

impl Eval for TypeUniv {
    type Val = Rc<Val>;

    fn eval(&self, _prg: &Module, _env: &mut Env) -> Result<Self::Val, TypeError> {
        let TypeUniv { span } = self;
        Ok(Rc::new(val::TypeUniv { span: *span }.into()))
    }
}

impl Eval for LocalMatch {
    type Val = Rc<Val>;

    /// Evaluate a LocalMatch:
    ///
    /// ```text
    /// e.match { ... }
    /// ┳        ━━┳━━
    /// ┃          ┗━━━━ cases
    /// ┗━━━━━━━━━━━━━━━ on_exp
    /// ```
    fn eval(&self, prg: &Module, env: &mut Env) -> Result<Self::Val, TypeError> {
        let LocalMatch { name: match_name, on_exp, cases, .. } = self;
        // We first evaluate `on_exp` and `cases`
        let on_exp = on_exp.eval(prg, env)?;
        let cases = cases.eval(prg, env)?;

        match (*on_exp).clone() {
            Val::Call(val::Call { name: ctor_name, args, .. }) => {
                // The specific instance of the LocalMatch we are evaluating is:
                //
                // ```text
                // C(e_1,...).match { ... }
                // ┳ ━━━┳━━━         ━━┳━━
                // ┃    ┃              ┗━━━━━ cases
                // ┃    ┗━━━━━━━━━━━━━━━━━━━━ args
                // ┗━━━━━━━━━━━━━━━━━━━━━━━━━ ctor__name
                // ```
                // where `C` is the name of a constructor declared in a data
                // type declaration.

                // We first look up the correct case.
                let val::Case { body, .. } =
                    cases.clone().into_iter().find(|case| case.name == ctor_name).unwrap();

                // Then we substitute the `args` in the body.
                body.clone().unwrap().apply(prg, &args)
            }
            Val::Neu(exp) => {
                // The specific instance of the LocalMatch we are evaluating is:
                //
                // ```text
                // n.match { ... }
                // ┳        ━━┳━━
                // ┃          ┗━━━━━ cases
                // ┗━━━━━━━━━━━━━━━━ exp (Neutral value)
                // ```
                // Evaluation is blocked by the neutral value `n`.
                Ok(Rc::new(Val::Neu(
                    val::LocalMatch {
                        span: None,
                        name: match_name.to_owned(),
                        on_exp: Rc::new(exp),
                        cases,
                    }
                    .into(),
                )))
            }
            _ => unreachable!(),
        }
    }
}

impl Eval for LocalComatch {
    type Val = Rc<Val>;

    fn eval(&self, prg: &Module, env: &mut Env) -> Result<Self::Val, TypeError> {
        let LocalComatch { span, name, is_lambda_sugar, cases, .. } = self;
        Ok(Rc::new(
            val::LocalComatch {
                span: *span,
                name: name.clone(),
                is_lambda_sugar: *is_lambda_sugar,
                cases: cases.eval(prg, env)?,
            }
            .into(),
        ))
    }
}

impl Eval for Hole {
    type Val = Rc<Val>;

    fn eval(&self, prg: &Module, env: &mut Env) -> Result<Self::Val, TypeError> {
        let Hole { span, metavar, args, .. } = self;
        let args = args.eval(prg, env)?;
        Ok(Rc::new(Val::Neu(val::Hole { span: *span, metavar: *metavar, args }.into())))
    }
}

impl Eval for Case {
    type Val = val::Case;

    fn eval(&self, _prg: &Module, env: &mut Env) -> Result<Self::Val, TypeError> {
        let Case { span, name, params, body } = self;

        let body = body.as_ref().map(|body| Closure {
            body: body.clone(),
            n_args: params.len(),
            env: env.clone(),
        });

        Ok(val::Case { span: *span, name: name.clone(), params: params.clone(), body })
    }
}

impl Eval for Args {
    type Val = Vec<Rc<Val>>;

    fn eval(&self, prg: &Module, env: &mut Env) -> Result<Self::Val, TypeError> {
        self.args.eval(prg, env)
    }
}

impl Apply for Closure {
    fn apply(mut self, prg: &Module, args: &[Rc<Val>]) -> Result<Rc<Val>, TypeError> {
        self.env.bind_iter(args.iter(), |env| self.body.eval(prg, env))
    }
}

impl<T: Eval> Eval for Vec<T> {
    type Val = Vec<T::Val>;

    fn eval(&self, prg: &Module, env: &mut Env) -> Result<Self::Val, TypeError> {
        self.iter().map(|x| x.eval(prg, env)).collect()
    }
}

impl Eval for Rc<Exp> {
    type Val = Rc<Val>;

    fn eval(&self, prg: &Module, env: &mut Env) -> Result<Self::Val, TypeError> {
        (**self).eval(prg, env)
    }
}
