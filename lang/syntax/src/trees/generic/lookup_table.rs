use std::fmt;

use crate::common::*;

#[derive(Clone, Copy, Debug)]
pub enum DeclKind {
    Data,
    Codata,
    Def,
    Codef,
    Ctor,
    Dtor,
}

impl fmt::Display for DeclKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeclKind::Data => write!(f, "data type"),
            DeclKind::Codata => write!(f, "codata type"),
            DeclKind::Def => write!(f, "definition"),
            DeclKind::Codef => write!(f, "codefinition"),
            DeclKind::Ctor => write!(f, "constructor"),
            DeclKind::Dtor => write!(f, "destructor"),
        }
    }
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

/// A top-level item in the source
#[derive(Debug, Clone)]
pub enum Item {
    Type(Type),
    Def(Def),
}

/// A type declaration in the source
#[derive(Debug, Clone)]
pub struct Type {
    pub name: Ident,
    pub xtors: Vec<Ident>,
}

impl Type {
    fn new(name: Ident) -> Self {
        Self { name, xtors: vec![] }
    }
}

/// A definition in the source
#[derive(Debug, Clone)]
pub struct Def {
    pub name: Ident,
}

impl Def {
    fn new(name: Ident) -> Self {
        Self { name }
    }
}

/// Metadata on declarations
#[derive(Debug, Clone, Default)]
pub struct LookupTable {
    /// List of items in the order they were defined in the source code
    items: Vec<Item>,
    /// Map of item names to the index in `items`
    item_idx: HashMap<Ident, usize>,
    /// Map of xtors to the type name they are associated to
    type_for_xtor: HashMap<Ident, Ident>,
    /// Map of xdefs to the type name they are associated to
    type_for_xdef: HashMap<Ident, Ident>,
    /// Map of type names to the xdefs that are associated to them
    xdefs_for_type: HashMap<Ident, HashSet<Ident>>,
}

impl LookupTable {
    pub fn get_or_add_type_decl(&mut self, name: Ident) -> TypeView<'_> {
        if self.item_idx.contains_key(&name) {
            self.type_decl_mut(&name).unwrap()
        } else {
            self.add_type_decl(name)
        }
    }

    pub fn add_type_decl(&mut self, name: Ident) -> TypeView<'_> {
        self.items.push(Item::Type(Type::new(name.clone())));
        self.item_idx.insert(name.clone(), self.items.len() - 1);
        self.type_decl_mut(&name).unwrap()
    }

    pub fn add_def(&mut self, type_name: Ident, def_name: Ident) {
        self.items.push(Item::Def(Def::new(def_name.clone())));
        self.type_for_xdef.insert(def_name.clone(), type_name.clone());
        self.xdefs_for_type.entry(type_name).or_default().insert(def_name);
    }

    /// Insert a definition after the corresponding type declaration
    pub fn insert_def(&mut self, type_name: Ident, def_name: Ident) {
        self.items.insert(self.item_idx[&type_name] + 1, Item::Def(Def::new(def_name.clone())));
        self.type_for_xdef.insert(def_name.clone(), type_name.clone());
        self.xdefs_for_type.entry(type_name).or_default().insert(def_name);
    }

    pub fn type_decl_mut(&mut self, name: &str) -> Option<TypeView<'_>> {
        self.type_decl(name)?;
        Some(TypeView { name: name.to_owned(), lookup_table: self })
    }

    pub fn iter(&self) -> impl Iterator<Item = &Item> {
        self.items.iter()
    }

    pub fn type_decl(&self, name: &str) -> Option<&Type> {
        let item = self.item_idx.get(name).and_then(|idx| self.items.get(*idx))?;
        let Item::Type(type_decl) = item else {
            return None;
        };
        Some(type_decl)
    }

    pub fn type_decl_for_xtor(&self, name: &str) -> Option<&Type> {
        let type_name = self.type_for_xtor.get(name)?;
        self.type_decl(type_name)
    }

    pub fn type_decl_for_xdef(&self, name: &str) -> Option<&Type> {
        let type_name = self.type_for_xdef.get(name)?;
        self.type_decl(type_name)
    }

    pub fn xtors_for_type(&self, name: &str) -> Vec<Ident> {
        self.type_decl(name).map(|decl| decl.xtors.clone()).unwrap_or_default()
    }

    pub fn xdefs_for_type(&self, name: &str) -> Vec<Ident> {
        match self.xdefs_for_type.get(name) {
            Some(set) => set.iter().cloned().collect(),
            None => vec![],
        }
    }

    fn type_raw_mut(&mut self, name: &str) -> Option<&mut Type> {
        let item = self.item_idx.get(name).and_then(|idx| self.items.get_mut(*idx))?;
        let Item::Type(type_decl) = item else {
            return None;
        };
        Some(type_decl)
    }
}
pub struct TypeView<'a> {
    name: Ident,
    lookup_table: &'a mut LookupTable,
}

impl<'a> TypeView<'a> {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn xtors(&self) -> &[Ident] {
        let type_decl = self.lookup_table.type_decl(&self.name).unwrap();
        &type_decl.xtors
    }

    pub fn set_xtors<I>(&mut self, xtors: I)
    where
        I: IntoIterator<Item = Ident>,
    {
        self.clear_xtors();
        for xtor in xtors {
            self.add_xtor(xtor);
        }
    }

    pub fn add_xtor(&mut self, xtor: Ident) {
        let type_decl = self.lookup_table.type_raw_mut(&self.name).unwrap();
        type_decl.xtors.push(xtor.clone());
        self.lookup_table.type_for_xtor.insert(xtor, self.name.clone());
    }

    pub fn clear_xtors(&mut self) {
        let type_decl = self.lookup_table.type_raw_mut(&self.name).unwrap();
        let xtors = std::mem::take(&mut type_decl.xtors);
        for xtor in &xtors {
            self.lookup_table.type_for_xtor.remove(xtor);
        }
    }
}
