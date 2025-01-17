use miette_util::codespan::Span;
use miette_util::ToMiette;

use ast::ctx::{BindContext, LevelCtx};
use ast::{self, MetaVar, MetaVarKind, MetaVarState};
use ast::{HashMap, HashSet};
use ast::{Idx, Lvl};
use parser::cst::ident::Ident;
use url::Url;

use crate::symbol_table::SymbolTable;

use super::result::LoweringError;

pub struct Ctx {
    /// Tracking local binder names
    ///
    /// Used to convert names to De-Bruijn indices
    binders: LevelCtx,
    /// Metadata for top-level names
    pub symbol_table: SymbolTable,
    /// Counter for unique label ids
    next_label_id: usize,
    /// Set of user-annotated label names
    user_labels: HashSet<Ident>,
    /// Counter for unique meta variables
    next_meta_var: u64,
    /// Meta variables
    pub meta_vars: HashMap<MetaVar, MetaVarState>,
    /// URI of the current module
    pub uri: Url,
}

impl Ctx {
    pub fn empty(uri: Url, symbol_table: SymbolTable) -> Self {
        Self {
            binders: LevelCtx::empty(),
            symbol_table,
            next_label_id: 0,
            user_labels: HashSet::default(),
            next_meta_var: 0,
            meta_vars: HashMap::default(),
            uri,
        }
    }

    /// Lookup in the local variable context.
    pub fn lookup_local(&self, name: &Ident) -> Option<Idx> {
        for fst in (0..self.binders.len()).rev() {
            let inner = &self.binders.bound[fst];
            for snd in (0..inner.len()).rev() {
                let ast::VarBind::Var { id, .. } = &inner[snd] else {
                    continue;
                };
                if id == &name.id {
                    return Some(self.level_to_index(Lvl { fst, snd }));
                }
            }
        }
        None
    }

    pub fn unique_label(
        &mut self,
        user_name: Option<Ident>,
        info: &Span,
    ) -> Result<ast::Label, LoweringError> {
        if let Some(user_name) = &user_name {
            if self.symbol_table.lookup_exists(user_name) {
                return Err(LoweringError::LabelNotUnique {
                    name: user_name.id.to_owned(),
                    span: info.to_miette(),
                });
            }

            if self.lookup_local(user_name).is_some() {
                return Err(LoweringError::LabelShadowed {
                    name: user_name.id.to_owned(),
                    span: info.to_miette(),
                });
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
        Ok(ast::Label {
            id,
            user_name: user_name.map(|name| ast::IdBind { span: Some(name.span), id: name.id }),
        })
    }

    /// Convert the given De-Bruijn level to a De-Bruijn index
    fn level_to_index(&self, lvl: Lvl) -> Idx {
        let fst = self.binders.len() - 1 - lvl.fst;
        let snd = self.binders.bound[lvl.fst].len() - 1 - lvl.snd;
        Idx { fst, snd }
    }

    /// Create a fresh MetaVar which stands for an unkown term that
    /// we have to elaborate later. The generated fresh variable is
    /// also registered as unsolved.
    pub fn fresh_metavar(&mut self, span: Option<Span>, kind: MetaVarKind) -> MetaVar {
        let mv = MetaVar { span, id: self.next_meta_var, kind };
        self.next_meta_var += 1;
        let ctx = self.binders.clone();
        log::trace!("Created fresh metavariable: {} in context: {:?}", mv.id, ctx.bound);
        self.meta_vars.insert(mv, MetaVarState::Unsolved { ctx });
        mv
    }

    /// With every context Γ there is an associated substitution id_Γ which consists of
    /// all variables in Γ. This function computes the substitution id_Γ.
    /// This substitution is needed when lowering typed holes since they stand for unknown terms which
    /// could potentially use all variables in the context.
    pub fn subst_from_ctx(&self) -> Vec<Vec<Box<ast::Exp>>> {
        let mut args: Vec<Vec<Box<ast::Exp>>> = Vec::new();
        let mut curr_subst = Vec::new();

        for (fst, inner) in self.binders.iter().enumerate() {
            for (snd, name) in inner.iter().enumerate() {
                curr_subst.push(Box::new(
                    ast::Variable {
                        span: None,
                        idx: self.level_to_index(Lvl { fst, snd }),
                        name: name.clone().into(),
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

impl BindContext for Ctx {
    type Ctx = LevelCtx;

    fn ctx_mut(&mut self) -> &mut Self::Ctx {
        &mut self.binders
    }
}
