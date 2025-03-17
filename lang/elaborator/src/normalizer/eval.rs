use std::rc::Rc;

use log::trace;

use ast::*;
use ctx::values::Binder;
use miette_util::ToMiette;
use printer::types::Print;

use crate::normalizer::env::*;
use crate::normalizer::val::{self, Closure, Val};

use crate::{result::*, TypeInfoTable};

use super::val::AnnoNeu;

pub trait Eval {
    type Val;

    fn eval(&self, info_table: &Rc<TypeInfoTable>, env: &mut Env) -> TcResult<Self::Val>;
}

pub trait Apply {
    fn apply(
        self,
        info_table: &Rc<TypeInfoTable>,
        args: Vec<Binder<Box<Val>>>,
    ) -> TcResult<Box<Val>>;
}

impl Eval for Exp {
    type Val = Box<Val>;

    fn eval(&self, info_table: &Rc<TypeInfoTable>, env: &mut Env) -> TcResult<Self::Val> {
        let e = match self {
            Exp::Variable(e) => e.eval(info_table, env),
            Exp::TypCtor(e) => e.eval(info_table, env),
            Exp::Call(e) => e.eval(info_table, env),
            Exp::DotCall(e) => e.eval(info_table, env),
            Exp::Anno(e) => e.eval(info_table, env),
            Exp::TypeUniv(e) => e.eval(info_table, env),
            Exp::LocalMatch(e) => e.eval(info_table, env),
            Exp::LocalComatch(e) => e.eval(info_table, env),
            Exp::Hole(e) => e.eval(info_table, env),
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
    type Val = Box<Val>;

    fn eval(&self, _info_table: &Rc<TypeInfoTable>, env: &mut Env) -> TcResult<Self::Val> {
        let Variable { idx, .. } = self;
        Ok(env.lookup(*idx).content)
    }
}

impl Eval for TypCtor {
    type Val = Box<Val>;

    fn eval(&self, info_table: &Rc<TypeInfoTable>, env: &mut Env) -> TcResult<Self::Val> {
        let TypCtor { span, name, args } = self;
        Ok(Box::new(
            val::TypCtor { span: *span, name: name.clone(), args: args.eval(info_table, env)? }
                .into(),
        ))
    }
}

impl Eval for Call {
    type Val = Box<Val>;

    fn eval(&self, info_table: &Rc<TypeInfoTable>, env: &mut Env) -> TcResult<Self::Val> {
        let Call { span, name, kind, args, .. } = self;
        match kind {
            CallKind::LetBound => {
                let Let { attr, body, params, .. } = info_table.lookup_let(name)?;
                // We now have to distinguish two cases:
                // If the let-bound definition is transparent, then we substitute the
                // arguments for the body of the definition. If it is opaque, then
                // the further computation is blocked so we return a neutral value.
                if attr.attrs.contains(&Attribute::Transparent) {
                    let args = args.eval(info_table, env)?;
                    let binders = params
                        .params
                        .iter()
                        .zip(args.to_vals())
                        .map(|(param, arg)| Binder { name: param.name.clone(), content: arg });
                    env.bind_iter(binders, |env| body.eval(info_table, env))
                } else {
                    Ok(Box::new(Val::Neu(
                        val::OpaqueCall {
                            span: *span,
                            name: name.clone(),
                            args: args.eval(info_table, env)?,
                        }
                        .into(),
                    )))
                }
            }
            CallKind::Constructor | CallKind::Codefinition => Ok(Box::new(
                val::Call {
                    span: *span,
                    kind: *kind,
                    name: name.clone(),
                    args: args.eval(info_table, env)?,
                }
                .into(),
            )),
        }
    }
}

impl Eval for DotCall {
    type Val = Box<Val>;

    /// Evaluate a DotCall:
    ///
    /// ```text
    /// e.d(e_1,...)
    /// ┳ ┳ ━━━┳━━━
    /// ┃ ┃    ┗━━━━━━━ args
    /// ┃ ┗━━━━━━━━━━━━ name
    /// ┗━━━━━━━━━━━━━━ exp
    /// ```
    fn eval(&self, info_table: &Rc<TypeInfoTable>, env: &mut Env) -> TcResult<Self::Val> {
        let DotCall { span, kind, exp, name, args, .. } = self;

        // We first evaluate `exp` and then the arguments `args` to `d` from left to right.
        let exp = exp.eval(info_table, env)?;
        let args = args.eval(info_table, env)?;

        // If possible, strip away all annotations from the expression.
        // For example, we need to strip away the annotation around `T` in  `(T : Bool).match { T => F, F => T }` before we can evaluate further.
        let exp = strip_annotations(&exp);

        match exp {
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
                        let Def { cases, params, .. } = info_table.lookup_def(&name.clone())?;
                        let mut env = Env::empty();
                        let binders = params
                            .params
                            .iter()
                            .zip(args.to_vals())
                            .map(|(param, arg)| Binder { name: param.name.clone(), content: arg });
                        let cases = env.bind_iter(binders, |env| cases.eval(info_table, env))?;
                        let val::Case { body, params, .. } = cases
                            .clone()
                            .into_iter()
                            .find(|case| case.name == call_name)
                            .ok_or_else(|| TypeError::MissingCase { name: call_name.id.clone() })?;

                        // Then we apply the body to the `call_args`.
                        let binders = params
                            .params
                            .iter()
                            .zip(call_args.to_vals())
                            .map(|(param, arg)| Binder { name: param.name.clone(), content: arg })
                            .collect::<Vec<_>>();
                        body.clone().unwrap().apply(info_table, binders)
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
                        let Codef { cases, params, .. } =
                            info_table.lookup_codef(&call_name.clone())?;
                        let mut env = Env::empty();
                        let binders =
                            params.params.iter().zip(call_args.to_vals()).map(|(param, arg)| {
                                Binder { name: param.name.clone(), content: arg }
                            });
                        let cases = env.bind_iter(binders, |env| cases.eval(info_table, env))?;
                        let val::Case { body, params, .. } = cases
                            .clone()
                            .into_iter()
                            .find(|cocase| cocase.name == *name)
                            .ok_or_else(|| TypeError::MissingCocase { name: name.id.clone() })?;

                        // Then we apply the body to the `args`.
                        let binders = params
                            .params
                            .iter()
                            .zip(args.to_vals())
                            .map(|(param, arg)| Binder { name: param.name.clone(), content: arg })
                            .collect::<Vec<_>>();
                        body.clone().unwrap().apply(info_table, binders)
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
                let val::Case { body, params, .. } = cases
                    .clone()
                    .into_iter()
                    .find(|cocase| cocase.name == *name)
                    .ok_or_else(|| TypeError::MissingCocase { name: name.id.clone() })?;

                // Then we apply the body to the `args`.
                let binders = params
                    .params
                    .iter()
                    .zip(args.to_vals())
                    .map(|(param, arg)| Binder { name: param.name.clone(), content: arg })
                    .collect::<Vec<_>>();
                body.clone().unwrap().apply(info_table, binders)
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
                Ok(Box::new(Val::Neu(
                    val::DotCall {
                        span: *span,
                        kind: *kind,
                        exp: Box::new(exp),
                        name: name.to_owned(),
                        args,
                    }
                    .into(),
                )))
            }
            Val::Anno(_) => Err(TypeError::Impossible {
                message: "Type annotation was not stripped when evaluating DotCall".to_owned(),
                span: span.to_miette(),
            }
            .into()),
            Val::TypCtor(_) => Err(TypeError::Impossible {
                message: "Cannot apply DotCall to type constructor".to_owned(),
                span: span.to_miette(),
            }
            .into()),
            Val::TypeUniv(_) => Err(TypeError::Impossible {
                message: "Cannot apply DotCall to type universe".to_owned(),
                span: span.to_miette(),
            }
            .into()),
        }
    }
}

/// Given a value, strip away all the annotations and return the inner value.
/// Unless the inner value is neutral, in which case all annotations become neutral.
/// For example, stripping the annotations from `((T : Bool): Bool)` would yield `T` because `T` is not neutral.
/// Stripping the annotations from `((x: Bool): Bool)` would yield `((x: Bool): Bool)` because `x` is neutral.
fn strip_annotations(val: &Val) -> Val {
    match val {
        Val::Anno(anno) => match strip_annotations(&anno.exp) {
            Val::Neu(neu) => Val::Neu(
                AnnoNeu { span: anno.span, exp: Box::new(neu), typ: anno.typ.clone() }.into(),
            ),
            val => val,
        },
        val => val.clone(),
    }
}

impl Eval for Anno {
    type Val = Box<Val>;

    fn eval(&self, info_table: &Rc<TypeInfoTable>, env: &mut Env) -> TcResult<Self::Val> {
        let Anno { span, exp, typ, normalized_type: _ } = self;
        let exp = exp.eval(info_table, env)?;
        let typ = typ.eval(info_table, env)?;
        Ok(Box::new(val::AnnoVal { span: *span, exp, typ }.into()))
    }
}

impl Eval for TypeUniv {
    type Val = Box<Val>;

    fn eval(&self, _info_table: &Rc<TypeInfoTable>, _env: &mut Env) -> TcResult<Self::Val> {
        let TypeUniv { span } = self;
        Ok(Box::new(val::TypeUniv { span: *span }.into()))
    }
}

impl Eval for LocalMatch {
    type Val = Box<Val>;

    /// Evaluate a LocalMatch:
    ///
    /// ```text
    /// e.match { ... }
    /// ┳        ━━┳━━
    /// ┃          ┗━━━━ cases
    /// ┗━━━━━━━━━━━━━━━ on_exp
    /// ```
    fn eval(&self, info_table: &Rc<TypeInfoTable>, env: &mut Env) -> TcResult<Self::Val> {
        let LocalMatch { name: match_name, on_exp, cases, .. } = self;
        // We first evaluate `on_exp` and `cases`
        let on_exp = on_exp.eval(info_table, env)?;
        let cases = cases.eval(info_table, env)?;

        let on_exp = strip_annotations(&on_exp);

        match on_exp {
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
                let val::Case { body, params, .. } = cases
                    .clone()
                    .into_iter()
                    .find(|case| case.name == ctor_name)
                    .ok_or_else(|| TypeError::MissingCase { name: ctor_name.id.clone() })?;

                // Then we substitute the `args` in the body.
                let binders = params
                    .params
                    .iter()
                    .zip(args.to_vals())
                    .map(|(param, arg)| Binder { name: param.name.clone(), content: arg })
                    .collect::<Vec<_>>();
                body.clone().unwrap().apply(info_table, binders)
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
                Ok(Box::new(Val::Neu(
                    val::LocalMatch {
                        span: None,
                        name: match_name.to_owned(),
                        on_exp: Box::new(exp),
                        cases,
                    }
                    .into(),
                )))
            }
            Val::TypCtor(typ_ctor) => Err(TypeError::Impossible {
                message: "Cannot match on a type constructor".to_owned(),
                span: typ_ctor.span.to_miette(),
            }
            .into()),
            Val::TypeUniv(type_univ) => Err(TypeError::Impossible {
                message: "Cannot match on a type universe".to_owned(),
                span: type_univ.span.to_miette(),
            }
            .into()),
            Val::LocalComatch(local_comatch) => Err(TypeError::Impossible {
                message: "Cannot match on a local comatch".to_owned(),
                span: local_comatch.span.to_miette(),
            }
            .into()),
            Val::Anno(anno_val) => Err(TypeError::Impossible {
                message: "Type annotation was not stripped when evaluating local match".to_owned(),
                span: anno_val.span.to_miette(),
            }
            .into()),
        }
    }
}

impl Eval for Cases {
    type Val = ();

    fn eval(&self, info_table: &Rc<TypeInfoTable>, env: &mut Env) -> TcResult<Self::Val> {
        todo!()
    }
}

impl Eval for LocalComatch {
    type Val = Box<Val>;

    fn eval(&self, info_table: &Rc<TypeInfoTable>, env: &mut Env) -> TcResult<Self::Val> {
        let LocalComatch { span, name, is_lambda_sugar, cases, .. } = self;
        Ok(Box::new(
            val::LocalComatch {
                span: *span,
                name: name.clone(),
                is_lambda_sugar: *is_lambda_sugar,
                cases: cases.eval(info_table, env)?,
            }
            .into(),
        ))
    }
}

impl Eval for Hole {
    type Val = Box<Val>;

    fn eval(&self, info_table: &Rc<TypeInfoTable>, env: &mut Env) -> TcResult<Self::Val> {
        let Hole { span, kind, metavar, args, solution, .. } = self;
        let args = args.eval(info_table, env)?;
        let solution = solution.eval(info_table, env)?;
        Ok(Box::new(Val::Neu(
            val::Hole { span: *span, kind: *kind, metavar: *metavar, args, solution }.into(),
        )))
    }
}

impl<T: Eval> Eval for Binder<T> {
    type Val = Binder<T::Val>;

    fn eval(&self, info_table: &Rc<TypeInfoTable>, env: &mut Env) -> TcResult<Self::Val> {
        let Binder { name, content } = self;

        Ok(Binder { name: name.to_owned(), content: content.eval(info_table, env)? })
    }
}

impl Eval for Case {
    type Val = val::Case;

    fn eval(&self, _info_table: &Rc<TypeInfoTable>, env: &mut Env) -> TcResult<Self::Val> {
        let Case { span, pattern, body } = self;

        let body = body.as_ref().map(|body| Closure {
            body: body.clone(),
            params: pattern.params.params.iter().map(|p| p.name.clone()).collect(),
            env: env.clone(),
        });

        Ok(val::Case {
            span: *span,
            is_copattern: pattern.is_copattern,
            name: pattern.name.clone(),
            params: pattern.params.clone(),
            body,
        })
    }
}

impl Eval for Args {
    type Val = val::Args;

    fn eval(&self, info_table: &Rc<TypeInfoTable>, env: &mut Env) -> TcResult<Self::Val> {
        Ok(val::Args(self.args.eval(info_table, env)?))
    }
}

impl Eval for Arg {
    type Val = val::Arg;

    fn eval(&self, info_table: &Rc<TypeInfoTable>, env: &mut Env) -> TcResult<Self::Val> {
        match self {
            Arg::UnnamedArg { arg, .. } => Ok(val::Arg::UnnamedArg(arg.eval(info_table, env)?)),
            Arg::NamedArg { name, arg, .. } => {
                Ok(val::Arg::NamedArg(name.clone(), arg.eval(info_table, env)?))
            }
            Arg::InsertedImplicitArg { hole, .. } => {
                Ok(val::Arg::InsertedImplicitArg(hole.eval(info_table, env)?))
            }
        }
    }
}

impl Apply for Closure {
    fn apply(
        mut self,
        info_table: &Rc<TypeInfoTable>,
        args: Vec<Binder<Box<Val>>>,
    ) -> TcResult<Box<Val>> {
        self.env.bind_iter(args.into_iter(), |env| self.body.eval(info_table, env))
    }
}

impl<T: Eval> Eval for Vec<T> {
    type Val = Vec<T::Val>;

    fn eval(&self, info_table: &Rc<TypeInfoTable>, env: &mut Env) -> TcResult<Self::Val> {
        self.iter().map(|x| x.eval(info_table, env)).collect()
    }
}

impl Eval for Box<Exp> {
    type Val = Box<Val>;

    fn eval(&self, info_table: &Rc<TypeInfoTable>, env: &mut Env) -> TcResult<Self::Val> {
        (**self).eval(info_table, env)
    }
}

impl<T: Eval> Eval for Option<T> {
    type Val = Option<T::Val>;

    fn eval(&self, info_table: &Rc<TypeInfoTable>, env: &mut Env) -> TcResult<Self::Val> {
        self.as_ref().map(|x| x.eval(info_table, env)).transpose()
    }
}
