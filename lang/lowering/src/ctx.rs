use std::rc::Rc;

use codespan::Span;
use miette_util::ToMiette;
use parser::cst;

use parser::cst::exp::BindingSite;
use parser::cst::ident::Ident;
use syntax::ast::lookup_table::DeclMeta;
use syntax::ast::{self, MetaVar, MetaVarState};
use syntax::ast::{HasSpan, Named};
use syntax::ast::{HashMap, HashSet};
use syntax::ast::{Idx, Lvl};
use syntax::ctx::{Context, ContextElem, LevelCtx};

use super::result::LoweringError;

pub struct Ctx {
    /// Map that resolves local binder names to De-Bruijn levels
    ///
    /// For each name, store a vector representing the different binders
    /// represented by this name. The last entry represents the binder currently in scope,
    /// the remaining entries represent the binders which are currently shadowed.
    ///
    /// Bound variables in this map are De-Bruijn levels rather than indices:
    local_map: HashMap<Ident, Vec<Lvl>>,
    /// Metadata for top-level names
    top_level_map: HashMap<Ident, DeclMeta>,
    /// Accumulates top-level declarations
    pub decls_map: HashMap<String, ast::Decl>,
    /// Counts the number of entries for each De-Bruijn level
    levels: Vec<usize>,
    /// Counter for unique label ids
    next_label_id: usize,
    /// Set of user-annotated label names
    user_labels: HashSet<Ident>,
    /// Counter for unique meta variables
    next_meta_var: u64,
    /// Meta variables
    pub meta_vars: HashMap<MetaVar, MetaVarState>,
}

impl Ctx {
    pub fn empty(top_level_map: HashMap<Ident, DeclMeta>) -> Self {
        Self {
            local_map: HashMap::default(),
            top_level_map,
            decls_map: HashMap::default(),
            levels: Vec::new(),
            next_label_id: 0,
            user_labels: HashSet::default(),
            next_meta_var: 0,
            meta_vars: HashMap::default(),
        }
    }

    /// Lookup in the local variable context.
    pub fn lookup_local(&self, name: &Ident) -> Option<Lvl> {
        self.lookup(name.clone())
    }

    /// Lookup in the global context of declarations.
    pub fn lookup_global(&self, name: &Ident) -> Option<&DeclMeta> {
        self.top_level_map.get(name)
    }

    pub fn lookup_top_level_decl(
        &self,
        name: &Ident,
        info: &Span,
    ) -> Result<DeclMeta, LoweringError> {
        self.top_level_map.get(name).cloned().ok_or_else(|| LoweringError::UndefinedIdent {
            name: name.to_owned(),
            span: info.to_miette(),
        })
    }

    pub fn add_decls<I>(&mut self, decls: I) -> Result<(), LoweringError>
    where
        I: IntoIterator<Item = ast::Decl>,
    {
        decls.into_iter().try_for_each(|decl| self.add_decl(decl))
    }

    pub fn add_decl(&mut self, decl: ast::Decl) -> Result<(), LoweringError> {
        if self.decls_map.contains_key(&decl.name().clone()) {
            return Err(LoweringError::AlreadyDefined {
                name: Ident { id: decl.name().clone() },
                span: decl.span().to_miette(),
            });
        }
        self.decls_map.insert(decl.name().clone(), decl);
        Ok(())
    }

    pub fn unique_label(
        &mut self,
        user_name: Option<Ident>,
        info: &Span,
    ) -> Result<ast::Label, LoweringError> {
        if let Some(user_name) = &user_name {
            match self.lookup_global(user_name) {
                Some(_) => {
                    return Err(LoweringError::LabelNotUnique {
                        name: user_name.id.to_owned(),
                        span: info.to_miette(),
                    })
                }
                None => (),
            }
            match self.lookup_local(user_name) {
                Some(_) => {
                    return Err(LoweringError::LabelShadowed {
                        name: user_name.id.to_owned(),
                        span: info.to_miette(),
                    })
                }
                None => (),
            }
            if self.user_labels.contains(user_name) {
                return Err(LoweringError::LabelNotUnique {
                    name: user_name.id.to_owned(),
                    span: info.to_miette(),
                });
            }
            self.user_labels.insert(user_name.to_owned());
        }
        let id = self.next_label_id;
        self.next_label_id += 1;
        Ok(ast::Label { id, user_name: user_name.map(|name| name.id) })
    }

    /// Next De Bruijn level to be assigned
    fn curr_lvl(&self) -> Lvl {
        let fst = self.levels.len() - 1;
        let snd = *self.levels.last().unwrap_or(&0);
        Lvl { fst, snd }
    }

    /// Convert the given De-Bruijn level to a De-Bruijn index
    pub fn level_to_index(&self, lvl: Lvl) -> Idx {
        let fst = self.levels.len() - 1 - lvl.fst;
        let snd = self.levels[lvl.fst] - 1 - lvl.snd;
        Idx { fst, snd }
    }

    /// Create a fresh MetaVar which stands for an unkown term that
    /// we have to elaborate later. The generated fresh variable is
    /// also registered as unsolved.
    pub fn fresh_metavar(&mut self) -> MetaVar {
        let mv = MetaVar { id: self.next_meta_var };
        let ctx = LevelCtx { bound: self.levels.clone() };
        self.next_meta_var += 1;
        self.meta_vars.insert(mv, MetaVarState::Unsolved { ctx });
        mv
    }

    /// With every context Γ there is an associated substitution id_Γ which consists of
    /// all variables in Γ. This function computes the substitution id_Γ.
    /// This substitution is needed when lowering typed holes since they stand for unknown terms which
    /// could potentially use all variables in the context.
    pub fn subst_from_ctx(&self) -> Vec<Vec<Rc<ast::Exp>>> {
        let mut lvl_to_name: HashMap<Lvl, Ident> = HashMap::default();

        for (name, lvls) in self.local_map.iter() {
            for lvl in lvls {
                lvl_to_name.insert(*lvl, name.clone());
            }
        }

        let mut args: Vec<Vec<Rc<ast::Exp>>> = Vec::new();
        let mut curr_subst = Vec::new();

        for (fst, n) in self.levels.iter().cloned().enumerate() {
            for snd in 0..n {
                let lvl = Lvl { fst, snd };
                let name =
                    lvl_to_name.get(&lvl).map(|ident| ident.id.to_owned()).unwrap_or_default();
                curr_subst.push(Rc::new(
                    ast::Variable {
                        span: None,
                        idx: self.level_to_index(Lvl { fst, snd }),
                        name: name.to_owned(),
                        inferred_type: None,
                    }
                    .into(),
                ))
            }
            args.push(curr_subst);
            curr_subst = vec![];
        }

        args
    }
}

impl Context for Ctx {
    type ElemIn = Ident;

    type ElemOut = Option<Lvl>;

    type Var = Ident;

    fn push_telescope(&mut self) {
        self.levels.push(0);
    }

    fn pop_telescope(&mut self) {
        self.levels.pop().unwrap();
    }

    fn push_binder(&mut self, elem: Self::ElemIn) {
        let var = self.curr_lvl();
        *self.levels.last_mut().unwrap() += 1;
        let stack = self.local_map.entry(elem).or_default();
        stack.push(var);
    }

    fn pop_binder(&mut self, elem: Self::ElemIn) {
        let stack = self.local_map.get_mut(&elem).expect("Tried to read unknown variable");
        stack.pop().unwrap();
        *self.levels.last_mut().unwrap() -= 1;
    }

    fn lookup<V: Into<Self::Var>>(&self, var: V) -> Self::ElemOut {
        let name = var.into();
        self.local_map.get(&name).and_then(|stack| stack.last().cloned())
    }
}

impl ContextElem<Ctx> for Ident {
    fn as_element(&self) -> <Ctx as Context>::ElemIn {
        self.to_owned()
    }
}

impl ContextElem<Ctx> for &cst::decls::Param {
    fn as_element(&self) -> <Ctx as Context>::ElemIn {
        match &self.name {
            BindingSite::Var { name, .. } => name.to_owned(),
            BindingSite::Wildcard { .. } => Ident { id: "_".to_owned() },
        }
    }
}

impl ContextElem<Ctx> for &cst::exp::BindingSite {
    fn as_element(&self) -> <Ctx as Context>::ElemIn {
        match self {
            BindingSite::Var { name, .. } => name.to_owned(),
            BindingSite::Wildcard { .. } => Ident { id: "_".to_owned() },
        }
    }
}
