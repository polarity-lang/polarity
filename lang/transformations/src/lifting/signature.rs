use std::cmp;

use ast::ctx::*;
use ast::*;
use ast::{Occurs, Variable};

use super::fv::*;

/// Compute the lifted signature based on the set of free variables of an expression `e`.
/// Using the lifted signature `LiftedSignature { telescope, subst, args }`, the lifted expression
///
/// ```text
/// let f(telescope) { e[subst] }
/// ```
///
/// can be constructed, where `f` is a fresh name. The expression `e` can be replaced by `f(args)`.
///
/// # Parameters
///
/// - `fvs`: Set of free variables
/// - `base_ctx`: Context under which the expression `e` is well-typed
///
/// # Requires
///
/// - `fvs ⊆ base_ctx`
///
/// # Returns
///
/// The signature `LiftedSignature { telescope, subst, args }` of the lifted expression, consisting of:
///
/// - `telescope`: The telescope under which `e[subst]` is closed
/// - `subst`: A substitution that closes the free variables under `telescope`
/// - `args`: The arguments to apply to the lifted expression
///
/// # Ensures
///
/// - `telescope ⊆ base_ctx`
/// - `let f(telescope) { e[subst] }` is well-typed in the empty context
/// - `e = f(args)` and well-typed in `base_ctx`
///
pub fn lifted_signature(fvs: HashSet<FreeVar>, base_ctx: &LevelCtx) -> LiftedSignature {
    let cutoff = base_ctx.len();
    let fvs = sort_free_vars(fvs);

    let mut params: Vec<Param> = vec![];
    let mut args = vec![];
    let mut free_vars = FreeVarsMap::new(cutoff);

    for fv in fvs.into_iter() {
        let FreeVar { name, lvl, typ, ctx } = fv;

        let typ = typ.subst(&ctx, &free_vars.to_param_subst());

        let param = Param {
            implicit: false,
            name: VarBind::from_string(&name),
            typ: typ.clone(),
            erased: false,
        };
        let arg = Arg::UnnamedArg {
            arg: Box::new(Exp::Variable(Variable {
                span: None,
                idx: base_ctx.lvl_to_idx(lvl),
                name: VarBound::from_string(&name),
                inferred_type: None,
            })),
            erased: false,
        };
        args.push(arg);
        params.push(param);
        free_vars.add(name, lvl);
    }

    LiftedSignature {
        telescope: Telescope { params },
        subst: free_vars.into_body_subst(base_ctx),
        args: Args { args },
    }
}

/// The signature of a lifted expression
pub struct LiftedSignature {
    /// Telescope of the lifted expression
    pub telescope: Telescope,
    /// Substitution that is applied to the body of the lifted expression
    pub subst: Subst,
    /// An instantiation of `telescope` with the free variables
    pub args: Args,
}

/// Sort the free variables such that the dependency relation is satisfied
fn sort_free_vars(fvs: HashSet<FreeVar>) -> Vec<FreeVar> {
    let mut fvs: Vec<_> = fvs.into_iter().collect();
    fvs.sort();
    fvs
}

impl PartialOrd for FreeVar {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FreeVar {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let self_occurs_in_other = other.typ.occurs_var(&mut other.ctx.clone(), self.lvl);
        let other_occurs_in_self = self.typ.occurs_var(&mut self.ctx.clone(), other.lvl);
        assert!(!(self_occurs_in_other && other_occurs_in_self));
        if self_occurs_in_other {
            cmp::Ordering::Less
        } else if other_occurs_in_self {
            cmp::Ordering::Greater
        } else {
            self.lvl.cmp(&other.lvl)
        }
    }
}

/// Map from level of free variables to new to-be-substituted bound variables
#[derive(Clone, Debug)]
pub struct FreeVarsMap {
    map: HashMap<Lvl, NewBoundVar>,
    cutoff: usize,
}

#[derive(Clone, Debug)]
struct NewBoundVar {
    name: String,
    lvl: Lvl,
}

impl FreeVarsMap {
    fn new(cutoff: usize) -> Self {
        Self { map: Default::default(), cutoff }
    }

    fn add(&mut self, name: String, lvl: Lvl) {
        self.map.insert(lvl, NewBoundVar { name, lvl: Lvl { fst: 0, snd: self.map.len() } });
    }

    fn to_param_subst(&self) -> Subst {
        let n = self.map.len();
        let mut map = HashMap::default();
        for (orig_lvl, nv) in &self.map {
            let idx = Idx { fst: 0, snd: n - 1 - nv.lvl.snd };
            map.insert(
                *orig_lvl,
                Exp::Variable(Variable {
                    span: None,
                    idx,
                    name: VarBound::from_string(&nv.name),
                    inferred_type: None,
                }),
            );
        }
        Subst { map }
    }

    fn into_body_subst(self, ctx: &LevelCtx) -> Subst {
        let mut pairs: Vec<_> = self.map.iter().collect();
        pairs.sort_by_key(|(lvl, _)| **lvl);
        let free_vars = pairs
            .into_iter()
            .map(|(_, var)| VarBind::Var { id: var.name.clone(), span: None })
            .collect::<Vec<_>>();
        let new_ctx = LevelCtx::from(vec![free_vars]).append(&ctx.tail(self.cutoff));

        let mut map = HashMap::default();
        for (orig_lvl, nv) in self.map.into_iter() {
            map.insert(
                orig_lvl,
                Exp::Variable(Variable {
                    span: None,
                    idx: new_ctx.lvl_to_idx(nv.lvl),
                    name: VarBound::from_string(&nv.name),
                    inferred_type: None,
                }),
            );
        }
        Subst { map }
    }
}
