use derivative::Derivative;

use ast::ctx::values::TypeCtx;
use ast::ctx::*;
use ast::*;
use ast::{Hole, TypeUniv, Variable};

use super::subst::*;

/// Find all free variables
pub fn free_vars<T: FV>(arg: &T, ctx: &TypeCtx) -> FreeVars {
    let mut v = FreeVarsVisitor {
        fvs: Default::default(),
        cutoff: ctx.len(),
        type_ctx: ctx,
        lvl_ctx: ctx.levels(),
    };

    arg.visit_fv(&mut v);

    FreeVars { fvs: v.fvs, cutoff: ctx.len() }
}

pub trait FV {
    fn visit_fv(&self, v: &mut FreeVarsVisitor);
}

impl FV for Vec<Case> {
    fn visit_fv(&self, v: &mut FreeVarsVisitor) {
        for case in self {
            case.visit_fv(v)
        }
    }
}

impl FV for Exp {
    fn visit_fv(&self, v: &mut FreeVarsVisitor) {
        match self {
            Exp::Anno(Anno { exp, typ, .. }) => {
                exp.visit_fv(v);
                typ.visit_fv(v)
            }
            Exp::Variable(e) => e.visit_fv(v),
            Exp::LocalComatch(LocalComatch { cases, .. }) => {
                for case in cases {
                    case.visit_fv(v)
                }
            }
            Exp::Call(Call { args, .. }) => args.visit_fv(v),
            Exp::DotCall(DotCall { exp, args, .. }) => {
                exp.visit_fv(v);
                args.visit_fv(v);
            }
            Exp::TypCtor(e) => e.visit_fv(v),
            Exp::Hole(e) => e.visit_fv(v),
            Exp::TypeUniv(TypeUniv { span: _ }) => {}
            Exp::LocalMatch(LocalMatch { on_exp, motive, cases, .. }) => {
                for case in cases {
                    case.visit_fv(v);
                }
                on_exp.visit_fv(v);
                motive.visit_fv(v)
            }
        }
    }
}

impl FV for Variable {
    fn visit_fv(&self, v: &mut FreeVarsVisitor) {
        let Variable { idx, name, .. } = self;
        // We use the level context to convert the De Bruijn index to a De Bruijn level
        let lvl = v.lvl_ctx.idx_to_lvl(*idx);
        // If the variable is considered free (based on the cutoff), we look up its type in the typing context
        // The typing context contains the types for all free variables where lvl < cutoff
        if lvl.fst < v.cutoff {
            let typ = shift_and_clone(
                &v.type_ctx.lookup(lvl).content,
                ((v.lvl_ctx.len() - v.type_ctx.len()) as isize, 0),
            );
            v.add_fv(name.id.clone(), lvl, typ, v.lvl_ctx.clone())
        }
    }
}

impl FV for TypCtor {
    fn visit_fv(&self, v: &mut FreeVarsVisitor) {
        let TypCtor { span: _, name: _, args } = self;
        args.visit_fv(v)
    }
}

impl FV for Args {
    fn visit_fv(&self, v: &mut FreeVarsVisitor) {
        let Args { args } = self;
        for arg in args {
            arg.visit_fv(v)
        }
    }
}

impl FV for Arg {
    fn visit_fv(&self, v: &mut FreeVarsVisitor) {
        match self {
            Arg::UnnamedArg { arg, .. } => arg.visit_fv(v),
            Arg::NamedArg { arg, .. } => arg.visit_fv(v),
            Arg::InsertedImplicitArg { hole, .. } => hole.visit_fv(v),
        }
    }
}

impl FV for Hole {
    fn visit_fv(&self, v: &mut FreeVarsVisitor) {
        let Hole { args, .. } = self;
        for subst in args {
            for exp in subst {
                exp.visit_fv(v);
            }
        }
    }
}

impl FV for Case {
    fn visit_fv(&self, v: &mut FreeVarsVisitor) {
        let Case { span: _, pattern, body } = self;

        v.bind_iter(pattern.params.params.iter(), |v| {
            body.visit_fv(v);
        })
    }
}

impl FV for Motive {
    fn visit_fv(&self, v: &mut FreeVarsVisitor) {
        let Motive { span: _, param, ret_typ } = self;

        param.visit_fv(v);

        v.bind_single(param, |v| ret_typ.visit_fv(v))
    }
}

impl FV for ParamInst {
    fn visit_fv(&self, _v: &mut FreeVarsVisitor) {
        //contains no type info for ust.
    }
}

impl<T: FV> FV for Box<T> {
    fn visit_fv(&self, v: &mut FreeVarsVisitor) {
        (**self).visit_fv(v)
    }
}

impl<T: FV> FV for Option<T> {
    fn visit_fv(&self, v: &mut FreeVarsVisitor) {
        if let Some(x) = self {
            x.visit_fv(v)
        }
    }
}

/// Visitor that collects free variables
pub struct FreeVarsVisitor<'a> {
    /// Set of collected free variables
    fvs: HashSet<FreeVar>,
    /// The De-Bruijn level (fst index) up to which a variable counts as free
    cutoff: usize,
    /// The typing context where all free variables with lvl < cutoff can be looked up
    type_ctx: &'a TypeCtx,
    /// The level context which tracks the number of binders currently in scope
    lvl_ctx: LevelCtx,
}

impl FreeVarsVisitor<'_> {
    /// Add a free variable as well as all free variables its type
    fn add_fv(&mut self, name: String, lvl: Lvl, typ: Box<Exp>, ctx: LevelCtx) {
        // Add the free variable
        let fv = FreeVar { name, lvl, typ: typ.clone(), ctx };
        if self.fvs.insert(fv) {
            // If it has not already been added:
            // Find all free variables in the type of the free variable
            typ.visit_fv(self);
        }
    }
}

impl BindContext for FreeVarsVisitor<'_> {
    type Content = ();

    fn ctx_mut(&mut self) -> &mut LevelCtx {
        &mut self.lvl_ctx
    }
}

#[derive(Debug)]
pub struct FreeVars {
    /// The De-Bruijn level (fst index) up to which a variable counts as free
    pub(super) cutoff: usize,
    /// List of found free variables
    pub(super) fvs: HashSet<FreeVar>,
}

/// The result of closing under the set of free variables
pub struct FreeVarsResult {
    /// Telescope of the types of the free variables
    pub telescope: Telescope,
    /// A substitution close the free variables under `telescope`
    pub subst: FVSubst,
    /// An instantiation of `telescope` with the free variables
    pub args: Args,
}

#[derive(Clone, Debug, Derivative)]
#[derivative(Hash, Eq, PartialEq)]
pub struct FreeVar {
    /// Name of the free variable
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub name: String,
    /// The original De-Bruijn level
    pub lvl: Lvl,
    /// Type of the free variable
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub typ: Box<Exp>,
    /// Context under which the type is closed
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    pub ctx: LevelCtx,
}
