use codespan::Span;
use data::{HashMap, HashSet};
use miette_util::ToMiette;
use parser::cst;
use parser::cst::{BindingSite, Ident};
use syntax::common::*;
use syntax::ctx::{Context, ContextElem};
use syntax::generic::lookup_table;
use syntax::ust;

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
    decls_map: HashMap<Ident, ust::Decl>,
    /// Counts the number of entries for each De-Bruijn level
    levels: Vec<usize>,
    /// Counter for unique label ids
    next_label_id: usize,
    /// Set of user-annotated label names
    user_labels: HashSet<Ident>,
}

impl Ctx {
    pub fn lookup(&self, name: &str, info: &Span) -> Result<Elem, LoweringError> {
        Context::lookup(self, name.to_owned()).ok_or_else(|| LoweringError::UndefinedIdent {
            name: name.to_owned(),
            span: info.to_miette(),
        })
    }

    pub fn lookup_top_level_decl(
        &self,
        name: &str,
        info: &Span,
    ) -> Result<DeclMeta, LoweringError> {
        self.top_level_map.get(name).cloned().ok_or_else(|| LoweringError::UndefinedIdent {
            name: name.to_owned(),
            span: info.to_miette(),
        })
    }

    pub fn add_top_level_decl(
        &mut self,
        name: &Ident,
        decl_kind: DeclMeta,
    ) -> Result<(), LoweringError> {
        if self.top_level_map.contains_key(name) {
            return Err(LoweringError::AlreadyDefined { name: name.to_owned(), span: None });
        }
        self.top_level_map.insert(name.clone(), decl_kind);
        Ok(())
    }

    pub fn add_decls<I>(&mut self, decls: I) -> Result<(), LoweringError>
    where
        I: IntoIterator<Item = ust::Decl>,
    {
        decls.into_iter().try_for_each(|decl| self.add_decl(decl))
    }

    pub fn add_decl(&mut self, decl: ust::Decl) -> Result<(), LoweringError> {
        if self.decls_map.contains_key(decl.name()) {
            return Err(LoweringError::AlreadyDefined {
                name: decl.name().clone(),
                span: decl.span().to_miette(),
            });
        }
        self.decls_map.insert(decl.name().clone(), decl);
        Ok(())
    }

    pub fn into_decls(self, lookup_table: lookup_table::LookupTable) -> ust::Decls {
        ust::Decls { map: self.decls_map, lookup_table }
    }

    pub fn unique_label(
        &mut self,
        user_name: Option<Ident>,
        info: &Span,
    ) -> Result<Label, LoweringError> {
        if let Some(user_name) = &user_name {
            match Context::lookup(self, user_name) {
                Some(Elem::Decl(_)) => {
                    return Err(LoweringError::LabelNotUnique {
                        name: user_name.to_owned(),
                        span: info.to_miette(),
                    })
                }
                Some(Elem::Bound(_)) => {
                    return Err(LoweringError::LabelShadowed {
                        name: user_name.to_owned(),
                        span: info.to_miette(),
                    })
                }
                None => (),
            }
            if self.user_labels.contains(user_name) {
                return Err(LoweringError::LabelNotUnique {
                    name: user_name.to_owned(),
                    span: info.to_miette(),
                });
            }
            self.user_labels.insert(user_name.to_owned());
        }
        let id = self.next_label_id;
        self.next_label_id += 1;
        Ok(Label { id, user_name })
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
}

impl Context for Ctx {
    type ElemIn = Ident;

    type ElemOut = Option<Elem>;

    type Var = Ident;

    fn empty() -> Self {
        Self {
            local_map: HashMap::default(),
            top_level_map: HashMap::default(),
            decls_map: HashMap::default(),
            levels: Vec::new(),
            next_label_id: 0,
            user_labels: HashSet::default(),
        }
    }

    fn push_telescope(&mut self) {
        self.levels.push(0);
    }

    fn pop_telescope(&mut self) {
        self.levels.pop().unwrap();
    }

    fn push_binder(&mut self, elem: Self::ElemIn) {
        let var = self.curr_lvl();
        *self.levels.last_mut().unwrap() += 1;
        let stack = self.local_map.entry(elem).or_insert_with(Default::default);
        stack.push(var);
    }

    fn pop_binder(&mut self, elem: Self::ElemIn) {
        let stack = self.local_map.get_mut(&elem).expect("Tried to read unknown variable");
        stack.pop().unwrap();
        *self.levels.last_mut().unwrap() -= 1;
    }

    fn lookup<V: Into<Self::Var>>(&self, var: V) -> Self::ElemOut {
        let name = var.into();
        self.local_map
            .get(&name)
            .and_then(|stack| stack.last().cloned().map(Elem::Bound))
            .or_else(|| self.top_level_map.get(&name).cloned().map(Elem::Decl))
    }
}

impl ContextElem<Ctx> for Ident {
    fn as_element(&self) -> <Ctx as Context>::ElemIn {
        self.to_owned()
    }
}

impl ContextElem<Ctx> for &cst::Param {
    fn as_element(&self) -> <Ctx as Context>::ElemIn {
        match &self.name {
            BindingSite::Var { name } => name.to_owned(),
            BindingSite::Wildcard => "_".to_owned(),
        }
    }
}

impl ContextElem<Ctx> for &cst::ParamInst {
    fn as_element(&self) -> <Ctx as Context>::ElemIn {
        match &self.name {
            BindingSite::Var { name } => name.to_owned(),
            BindingSite::Wildcard => "_".to_owned(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Elem {
    Bound(Lvl),
    Decl(DeclMeta),
}

#[derive(Clone, Debug)]
pub enum DeclMeta {
    Data { arity: usize },
    Codata { arity: usize },
    Def,
    Codef,
    Ctor { ret_typ: Ident },
    Dtor { self_typ: Ident },
}

impl DeclMeta {
    pub fn kind(&self) -> DeclKind {
        match self {
            DeclMeta::Data { .. } => DeclKind::Data,
            DeclMeta::Codata { .. } => DeclKind::Codata,
            DeclMeta::Def => DeclKind::Def,
            DeclMeta::Codef => DeclKind::Codef,
            DeclMeta::Ctor { .. } => DeclKind::Ctor,
            DeclMeta::Dtor { .. } => DeclKind::Dtor,
        }
    }
}

impl From<&cst::Data> for DeclMeta {
    fn from(data: &cst::Data) -> Self {
        Self::Data { arity: data.params.len() }
    }
}

impl From<&cst::Codata> for DeclMeta {
    fn from(codata: &cst::Codata) -> Self {
        Self::Codata { arity: codata.params.len() }
    }
}

impl From<&cst::Def> for DeclMeta {
    fn from(_def: &cst::Def) -> Self {
        Self::Def
    }
}

impl From<&cst::Codef> for DeclMeta {
    fn from(_codef: &cst::Codef) -> Self {
        Self::Codef
    }
}

impl From<&cst::Item> for DeclMeta {
    fn from(decl: &cst::Item) -> Self {
        match decl {
            cst::Item::Data(data) => DeclMeta::from(data),
            cst::Item::Codata(codata) => DeclMeta::from(codata),
            cst::Item::Def(_def) => Self::Def,
            cst::Item::Codef(_codef) => Self::Codef,
        }
    }
}
