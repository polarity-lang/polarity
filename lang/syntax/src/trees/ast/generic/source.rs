use data::HashMap;

use crate::common::*;

/// Order in which declarations are defined in the source
#[derive(Debug, Clone, Default)]
pub struct Source {
    /// List of items in the order they were defined in the source code
    items: Vec<Item>,
    /// Map of type names to the index of the type declaration in `items`
    decl_for_type: HashMap<Ident, usize>,
    /// Map of type names to the index of the impl block in `items`
    impl_block_for_type: HashMap<Ident, usize>,
    /// Map of xtors to the type name they are associated to
    type_for_xtor: HashMap<Ident, Ident>,
    /// Map of xdefs to the type name they are associated to
    type_for_xdef: HashMap<Ident, Ident>,
}

impl Source {
    pub fn get_or_add_type_decl(&mut self, name: Ident) -> TypeView<'_> {
        if self.decl_for_type.contains_key(&name) {
            self.type_decl_mut(&name).unwrap()
        } else {
            self.add_type_decl(name)
        }
    }

    pub fn add_type_decl(&mut self, name: Ident) -> TypeView<'_> {
        self.items.push(Item::Type(Type::new(name.clone())));
        self.decl_for_type.insert(name.clone(), self.items.len() - 1);
        self.type_decl_mut(&name).unwrap()
    }

    pub fn get_or_add_impl_block(&mut self, name: Ident) -> ImplView<'_> {
        if self.impl_block_for_type.contains_key(&name) {
            self.impl_block_mut(&name).unwrap()
        } else {
            self.add_impl_block(name)
        }
    }

    pub fn add_impl_block(&mut self, name: Ident) -> ImplView<'_> {
        self.items.push(Item::Impl(Impl::new(name.clone())));
        self.impl_block_for_type.insert(name.clone(), self.items.len() - 1);
        self.impl_block_mut(&name).unwrap()
    }

    pub fn type_decl_mut(&mut self, name: &str) -> Option<TypeView<'_>> {
        self.type_decl(name)?;
        Some(TypeView { name: name.to_owned(), source: self })
    }

    pub fn impl_block_mut(&mut self, name: &str) -> Option<ImplView<'_>> {
        self.impl_block(name)?;
        Some(ImplView { name: name.to_owned(), source: self })
    }

    pub fn iter(&self) -> impl Iterator<Item = &Item> {
        self.items.iter()
    }

    pub fn type_decl(&self, name: &str) -> Option<&Type> {
        let item = self.decl_for_type.get(name).and_then(|idx| self.items.get(*idx))?;
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

    fn type_raw_mut(&mut self, name: &str) -> Option<&mut Type> {
        let item = self.decl_for_type.get(name).and_then(|idx| self.items.get_mut(*idx))?;
        let Item::Type(type_decl) = item else {
            return None;
        };
        Some(type_decl)
    }

    pub fn impl_block(&self, name: &str) -> Option<&Impl> {
        let item = self.impl_block_for_type.get(name).and_then(|idx| self.items.get(*idx))?;
        let Item::Impl(impl_block) = item else {
            return None;
        };
        Some(impl_block)
    }

    fn impl_raw_mut(&mut self, name: &str) -> Option<&mut Impl> {
        let item = self.impl_block_for_type.get(name).and_then(|idx| self.items.get_mut(*idx))?;
        let Item::Impl(impl_block) = item else {
            return None;
        };
        Some(impl_block)
    }
}

/// A top-level item in the source
#[derive(Debug, Clone)]
pub enum Item {
    Type(Type),
    Impl(Impl),
}

impl Named for Item {
    fn name(&self) -> &Ident {
        match self {
            Item::Type(Type { name, .. }) => name,
            Item::Impl(Impl { name, .. }) => name,
        }
    }
}

pub struct TypeView<'a> {
    name: Ident,
    source: &'a mut Source,
}

impl<'a> TypeView<'a> {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn xtors(&self) -> &[Ident] {
        let type_decl = self.source.type_decl(&self.name).unwrap();
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
        let type_decl = self.source.type_raw_mut(&self.name).unwrap();
        type_decl.xtors.push(xtor.clone());
        self.source.type_for_xtor.insert(xtor, self.name.clone());
    }

    pub fn clear_xtors(&mut self) {
        let type_decl = self.source.type_raw_mut(&self.name).unwrap();
        let xtors = std::mem::take(&mut type_decl.xtors);
        for xtor in &xtors {
            self.source.type_for_xtor.remove(xtor);
        }
    }
}

pub struct ImplView<'a> {
    name: Ident,
    source: &'a mut Source,
}

impl<'a> ImplView<'a> {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn defs(&self) -> &[Ident] {
        let impl_block = self.source.impl_block(&self.name).unwrap();
        &impl_block.defs
    }

    pub fn set_defs<I>(&mut self, defs: I)
    where
        I: IntoIterator<Item = Ident>,
    {
        self.clear_defs();
        for def in defs {
            self.add_def(def);
        }
    }

    pub fn add_def(&mut self, def: Ident) {
        let impl_block = self.source.impl_raw_mut(&self.name).unwrap();
        impl_block.defs.push(def.clone());
        self.source.type_for_xdef.insert(def, self.name.clone());
    }

    pub fn clear_defs(&mut self) {
        let impl_block = self.source.impl_raw_mut(&self.name).unwrap();
        let defs = std::mem::take(&mut impl_block.defs);
        for def in &defs {
            self.source.type_for_xdef.remove(def);
        }
    }
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

/// An impl block in the source
#[derive(Debug, Clone)]
pub struct Impl {
    pub name: Ident,
    pub defs: Vec<Ident>,
}

impl Impl {
    pub fn new(name: Ident) -> Self {
        Self { name, defs: vec![] }
    }
}
