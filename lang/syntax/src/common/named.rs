use super::*;
use crate::ast::generic;
use crate::cst;

pub trait Named {
    fn name(&self) -> &Ident;
}

impl Named for cst::Item {
    fn name(&self) -> &Ident {
        match self {
            cst::Item::Type(typ_decl) => typ_decl.name(),
            cst::Item::Def(def_decl) => def_decl.name(),
        }
    }
}

impl Named for cst::TypDecl {
    fn name(&self) -> &Ident {
        match self {
            cst::TypDecl::Data(cst::Data { name, .. }) => name,
            cst::TypDecl::Codata(cst::Codata { name, .. }) => name,
        }
    }
}

impl Named for cst::DefDecl {
    fn name(&self) -> &Ident {
        match self {
            cst::DefDecl::Def(cst::Def { name, .. }) => name,
            cst::DefDecl::Codef(cst::Codef { name, .. }) => name,
        }
    }
}

impl<P: generic::Phase> Named for generic::Decl<P> {
    fn name(&self) -> &Ident {
        match self {
            generic::Decl::Data(generic::Data { name, .. }) => name,
            generic::Decl::Codata(generic::Codata { name, .. }) => name,
            generic::Decl::Def(generic::Def { name, .. }) => name,
            generic::Decl::Codef(generic::Codef { name, .. }) => name,
            generic::Decl::Ctor(generic::Ctor { name, .. }) => name,
            generic::Decl::Dtor(generic::Dtor { name, .. }) => name,
        }
    }
}

impl Named for cst::Param {
    fn name(&self) -> &Ident {
        &self.name
    }
}

impl Named for cst::ParamInst {
    fn name(&self) -> &Ident {
        &self.name
    }
}

impl Named for Ident {
    fn name(&self) -> &Ident {
        self
    }
}

impl<'a, T> Named for &'a T
where
    T: Named,
{
    fn name(&self) -> &Ident {
        T::name(self)
    }
}

impl<P: generic::Phase> Named for generic::Param<P> {
    fn name(&self) -> &Ident {
        &self.name
    }
}

impl<P: generic::Phase> Named for generic::ParamInst<P> {
    fn name(&self) -> &Ident {
        &self.name
    }
}
