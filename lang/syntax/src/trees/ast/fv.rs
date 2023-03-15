use std::rc::Rc;

use derivative::Derivative;

use data::{HashMap, HashSet};

use crate::ast::generic::*;
use crate::common::*;
use crate::ctx::*;
use crate::wst;

/// Find all free variables
pub trait FreeVarsExt<P: Phase> {
    fn free_vars(&self, ctx: &mut TypeCtx<P>) -> FreeVars<P>;
}

#[derive(Debug)]
pub struct FreeVars<P: Phase> {
    /// The De-Bruijn level (fst index) up to which a variable counts as free
    cutoff: usize,
    /// List of found free variables
    fvs: HashSet<FreeVar<P>>,
}

/// The result of closing under the set of free variables
pub struct FreeVarsResult {
    /// Telescope of the types of the free variables
    pub telescope: wst::Telescope,
    /// A substitution close the free variables under `telescope`
    pub subst: FVSubst<wst::WST>,
    /// An instantiation of `telescope` with the free variables
    pub args: wst::Args,
}

impl FreeVars<wst::WST> {
    pub fn telescope(self) -> FreeVarsResult {
        let cutoff = self.cutoff;
        // Sort the list of free variables by the De-Bruijn level such the dependency relation is satisfied.
        // Types may be interdependent but can only depend on types which occur earlier in the context.
        let fvs = self.sorted();

        let mut params: wst::Params = vec![];
        let mut args: wst::Args = vec![];
        let mut subst = FVSubst::new(cutoff);

        // FIXME: The manual context management here should be abstracted out
        for fv in fvs.into_iter() {
            let FreeVar { name, lvl, typ, ctx } = fv;

            let typ = typ.subst(&mut ctx.levels(), &subst);

            let param = wst::Param { name: name.clone(), typ: typ.clone() };
            let info = wst::TypeInfo { typ: typ.forget(), span: Default::default() };
            let arg = Rc::new(wst::Exp::Var {
                info: info.clone(),
                name: name.clone(),
                idx: ctx.lvl_to_idx(fv.lvl),
            });
            for param in params.iter_mut() {
                param.typ = param.typ.shift((0, 1));
            }
            args.push(arg);
            params.push(param);
            subst.add(name, lvl, info);
        }

        FreeVarsResult { telescope: Telescope { params }, subst, args }
    }

    /// Compute the union of two free variable sets
    pub fn union(self, other: FreeVars<wst::WST>) -> FreeVars<wst::WST> {
        assert_eq!(self.cutoff, other.cutoff);
        let mut fvs = self.fvs;
        fvs.extend(other.fvs.into_iter());
        Self { fvs, cutoff: self.cutoff }
    }

    /// Sort the free variables according to their De-Bruijn level
    fn sorted(self) -> Vec<FreeVar<wst::WST>> {
        let mut fvs: Vec<_> = self.fvs.into_iter().collect();
        fvs.sort();
        fvs
    }
}

#[derive(Clone, Debug, Derivative)]
#[derivative(Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct FreeVar<P: Phase> {
    /// Name of the free variable
    #[derivative(PartialEq = "ignore", Hash = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    pub name: Ident,
    /// The original De-Bruijn level
    pub lvl: Lvl,
    /// Type of the free variable
    #[derivative(PartialEq = "ignore", Hash = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    pub typ: Rc<Exp<P>>,
    /// Context under which the type is closed
    #[derivative(PartialEq = "ignore", Hash = "ignore", PartialOrd = "ignore", Ord = "ignore")]
    pub ctx: TypeCtx<P>,
}

impl<P: Phase<VarName = Ident>, T: Visit<P>> FreeVarsExt<P> for T
where
    P::InfTyp: ShiftInRange,
    for<'b> &'b Param<P>: ContextElem<TypeCtx<P>>,
    for<'b> &'b ParamInst<P>: ContextElem<TypeCtx<P>>,
{
    fn free_vars(&self, ctx: &mut TypeCtx<P>) -> FreeVars<P> {
        let mut v = FvVistor { fvs: Default::default(), cutoff: ctx.len(), ctx };

        self.visit(&mut v);

        FreeVars { fvs: v.fvs, cutoff: ctx.len() }
    }
}

/// Visitor that collects free variables
struct FvVistor<'a, P: Phase> {
    /// Set of collected free variables
    fvs: HashSet<FreeVar<P>>,
    /// Current typing context
    ctx: &'a mut TypeCtx<P>,
    /// The De-Bruijn level (fst index) up to which a variable counts as free
    cutoff: usize,
}

impl<'a, P: Phase<VarName = Ident>> FvVistor<'a, P>
where
    P::InfTyp: ShiftInRange,
    for<'b> &'b Param<P>: ContextElem<TypeCtx<P>>,
    for<'b> &'b ParamInst<P>: ContextElem<TypeCtx<P>>,
{
    /// Add a free variable as well as all free variables its type
    fn add_fv(&mut self, fv: FreeVar<P>) {
        let typ = fv.typ.clone();
        // Add the free variable
        if self.fvs.insert(fv) {
            // If it has not already been added:
            // Find all free variables in the type of the free variable
            typ.visit(self);
        }
    }
}

impl<'a, P: Phase> BindContext for FvVistor<'a, P>
where
    P::InfTyp: ShiftInRange,
{
    type Ctx = TypeCtx<P>;

    fn ctx_mut(&mut self) -> &mut Self::Ctx {
        self.ctx
    }
}

impl<'b, P: Phase<VarName = Ident>> Visitor<P> for FvVistor<'b, P>
where
    P::InfTyp: ShiftInRange,
    for<'c> &'c Param<P>: ContextElem<TypeCtx<P>>,
    for<'c> &'c ParamInst<P>: ContextElem<TypeCtx<P>>,
{
    fn visit_telescope<'a, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2)
    where
        P: 'a,
        I: IntoIterator<Item = &'a Param<P>>,
        F1: Fn(&mut Self, &'a Param<P>),
        F2: FnOnce(&mut Self),
    {
        self.ctx_visit_telescope(params, f_acc, f_inner)
    }

    fn visit_telescope_inst<'a, I, F1, F2>(&mut self, params: I, f_acc: F1, f_inner: F2)
    where
        P: 'a,
        I: IntoIterator<Item = &'a ParamInst<P>>,
        F1: Fn(&mut Self, &'a ParamInst<P>),
        F2: FnOnce(&mut Self),
    {
        self.ctx_visit_telescope_inst(params, f_acc, f_inner)
    }

    fn visit_motive_param<X, F>(&mut self, param: &ParamInst<P>, f_inner: F) -> X
    where
        F: FnOnce(&mut Self, &ParamInst<P>) -> X,
    {
        self.ctx_visit_motive_param(param, f_inner)
    }

    fn visit_self_param<X, F>(
        &mut self,
        info: &<P as Phase>::Info,
        name: &Option<Ident>,
        typ: &TypApp<P>,
        f_inner: F,
    ) -> X
    where
        F: FnOnce(&mut Self) -> X,
    {
        self.ctx_visit_self_param(info, name, typ, f_inner)
    }

    fn visit_exp_var(&mut self, _info: &P::TypeInfo, name: &P::VarName, idx: &Idx) {
        let lvl = self.ctx.idx_to_lvl(*idx);
        // If the variable is considered free (based on the cutoff)
        if lvl.fst < self.cutoff {
            let typ = self.ctx.lookup(lvl);
            self.add_fv(FreeVar { name: name.clone(), lvl, typ, ctx: self.ctx.clone() });
        }
    }
}

/// Substitution of free variables
#[derive(Clone, Debug)]
pub struct FVSubst<P: Phase> {
    /// Mapping of the original De-Bruijn levels of a free variable to the new reference
    subst: HashMap<Lvl, NewVar<P>>,
    /// The De-Bruijn level (fst index) up to which a variable counts as free
    cutoff: usize,
}

impl<P: Phase> FVSubst<P> {
    fn new(cutoff: usize) -> Self {
        Self { subst: Default::default(), cutoff }
    }
}

/// A free variable as part of `FVSubst`
#[derive(Clone, Debug)]
struct NewVar<P: Phase> {
    /// Name of the free variable
    name: Ident,
    /// New De-Bruijn level
    lvl: Lvl,
    /// Type information of the variable
    info: P::TypeInfo,
}

impl<P: Phase> FVSubst<P> {
    fn add(&mut self, name: Ident, lvl: Lvl, info: P::TypeInfo) {
        self.subst.insert(lvl, NewVar { name, lvl: Lvl { fst: 0, snd: self.subst.len() }, info });
    }
}

impl<P: Phase> ShiftInRange for FVSubst<P> {
    fn shift_in_range<R: ShiftRange>(&self, _range: R, _by: (isize, isize)) -> Self {
        // Since FVSubst works with levels, it is shift-invariant
        self.clone()
    }
}

impl<P: Phase<VarName = Ident>> Substitution<Rc<Exp<P>>> for FVSubst<P> {
    fn get_subst(&self, ctx: &LevelCtx, lvl: Lvl) -> Option<Rc<Exp<P>>> {
        // Let Γ be the original context, let Δ be the context according to which the new De-Bruijn index should be calculated
        //
        // Γ = [[x], [y], [z]]
        //     ^^^^^^^^  ^
        //    free vars  cutoff
        //
        // Δ = [[x, y], [z]]
        //      ^^^^^^  ^^^ bound vars
        // new telescope
        let new_ctx = LevelCtx::from(vec![self.subst.len()]).append(&ctx.tail(self.cutoff));
        self.subst.get(&lvl).map(|fv| {
            Rc::new(Exp::Var {
                info: fv.info.clone(),
                name: fv.name.clone(),
                idx: new_ctx.lvl_to_idx(fv.lvl),
            })
        })
    }
}
