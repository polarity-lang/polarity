use data::HashMap;
use miette_util::ToMiette;
use syntax::ast::source;
use syntax::common::*;
use syntax::cst;
use syntax::ctx::Context;
use syntax::ust;

use super::result::LoweringError;

pub struct Ctx {
    /// For each name, store a vector representing the different binders
    /// represented by this name. The last entry represents the binder currently in scope,
    /// the remaining entries represent the binders which are currently shadowed.
    ///
    /// Bound variables in this map are De-Bruijn levels rather than indices:
    map: HashMap<Ident, Vec<Elem>>,
    /// Declaration metadata
    decl_kinds: HashMap<Ident, DeclKind>,
    /// Accumulates top-level declarations
    decls_map: HashMap<Ident, ust::Decl>,
    /// Counts the number of entries for each De-Bruijn level
    levels: Vec<usize>,
}

impl Ctx {
    pub fn lookup(&self, name: &str, info: &cst::Info) -> Result<&Elem, LoweringError> {
        self.map.get(name).and_then(|stack| stack.last()).ok_or_else(|| {
            LoweringError::UndefinedIdent { name: name.to_owned(), span: info.span.to_miette() }
        })
    }

    pub fn decl_kind(&self, name: &Ident) -> &DeclKind {
        &self.decl_kinds[name]
    }

    pub fn typ_name_for_xtor(&self, name: &Ident) -> &Ident {
        match &self.decl_kinds[name] {
            DeclKind::Ctor { ret_typ } => ret_typ,
            DeclKind::Dtor { self_typ } => self_typ,
            _ => panic!("Can only query type name for declared xtors"),
        }
    }

    pub fn typ_ctor_arity(&self, name: &Ident) -> usize {
        match self.decl_kind(name) {
            DeclKind::Data { arity } => *arity,
            DeclKind::Codata { arity } => *arity,
            _ => panic!("Can only query type constructor arity for declared (co)data types"),
        }
    }

    // FIXME: confusing name, rename or inline
    pub fn lower_bound(&self, lvl: Lvl) -> Idx {
        self.level_to_index(lvl)
    }

    pub fn add_name(&mut self, name: &Ident, decl_kind: DeclKind) -> Result<(), LoweringError> {
        self.decl_kinds.insert(name.clone(), decl_kind);
        let stack = self.map.entry(name.clone()).or_insert_with(Default::default);
        stack.push(Elem::Decl);
        Ok(())
    }

    pub fn add_decls<I>(&mut self, decls: I) -> Result<(), LoweringError>
    where
        I: IntoIterator<Item = ust::Decl>,
    {
        decls.into_iter().try_for_each(|decl| self.add_decl(decl))
    }

    pub fn add_decl(&mut self, decl: ust::Decl) -> Result<(), LoweringError> {
        match self.decls_map.get(decl.name()) {
            Some(_) => Err(LoweringError::AlreadyDefined {
                name: decl.name().clone(),
                span: decl.info().span.to_miette(),
            }),
            None => {
                self.decls_map.insert(decl.name().clone(), decl);
                Ok(())
            }
        }
    }

    pub fn into_decls(self, source: source::Source) -> ust::Decls {
        ust::Decls { map: self.decls_map, source }
    }

    /// Next De Bruijn level to be assigned
    fn curr_lvl(&self) -> Lvl {
        let fst = self.levels.len() - 1;
        let snd = *self.levels.last().unwrap_or(&0);
        Lvl { fst, snd }
    }

    /// Convert the given De-Bruijn level to a De-Bruijn index
    fn level_to_index(&self, lvl: Lvl) -> Idx {
        let fst = self.levels.len() - 1 - lvl.fst;
        let snd = self.levels[lvl.fst] - 1 - lvl.snd;
        Idx { fst, snd }
    }
}

impl Context for Ctx {
    type ElemIn = Ident;

    type ElemOut = Elem;

    type Var = Ident;

    fn empty() -> Self {
        Self {
            map: HashMap::default(),
            decl_kinds: HashMap::default(),
            decls_map: HashMap::default(),
            levels: Vec::new(),
        }
    }

    fn push_telescope(&mut self) {
        self.levels.push(0);
    }

    fn pop_telescope(&mut self) {
        self.levels.pop().unwrap();
    }

    fn push_binder(&mut self, elem: Self::ElemIn) {
        let var = Elem::Bound(self.curr_lvl());
        *self.levels.last_mut().unwrap() += 1;
        let stack = self.map.entry(elem).or_insert_with(Default::default);
        stack.push(var);
    }

    fn pop_binder(&mut self, elem: Self::ElemIn) {
        let stack = self.map.get_mut(&elem).expect("Tried to read unknown variable");
        stack.pop().unwrap();
        *self.levels.last_mut().unwrap() -= 1;
    }

    fn lookup<V: Into<Self::Var>>(&self, var: V) -> Self::ElemOut {
        let idx = var.into();
        self.map.get(&idx).and_then(|stack| stack.last()).cloned().unwrap()
    }
}

#[derive(Clone, Debug)]
pub enum Elem {
    Bound(Lvl),
    Decl,
}

// FIXME: Rename to DeclMeta or something similar
#[derive(Clone, Debug)]
pub enum DeclKind {
    Data { arity: usize },
    Codata { arity: usize },
    Def,
    Codef,
    Ctor { ret_typ: Ident },
    Dtor { self_typ: Ident },
}

impl From<&cst::Data> for DeclKind {
    fn from(data: &cst::Data) -> Self {
        Self::Data { arity: data.params.len() }
    }
}

impl From<&cst::Codata> for DeclKind {
    fn from(codata: &cst::Codata) -> Self {
        Self::Codata { arity: codata.params.len() }
    }
}

impl From<&cst::Def> for DeclKind {
    fn from(_def: &cst::Def) -> Self {
        Self::Def
    }
}

impl From<&cst::Codef> for DeclKind {
    fn from(_codef: &cst::Codef) -> Self {
        Self::Codef
    }
}

impl From<&cst::Item> for DeclKind {
    fn from(decl: &cst::Item) -> Self {
        match decl {
            cst::Item::Data(data) => DeclKind::from(data),
            cst::Item::Codata(codata) => DeclKind::from(codata),
            cst::Item::Def(_def) => Self::Def,
            cst::Item::Codef(_codef) => Self::Codef,
        }
    }
}
