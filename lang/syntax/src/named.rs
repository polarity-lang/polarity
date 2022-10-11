use super::ast;
use super::common::*;
use super::cst;

pub trait Named {
    fn name(&self) -> &Ident;
}

impl Named for cst::Item {
    fn name(&self) -> &Ident {
        match self {
            cst::Item::Type(typ_decl) => typ_decl.name(),
            cst::Item::Impl(cst::Impl { name, .. }) => name,
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

impl Named for ast::Decl {
    fn name(&self) -> &Ident {
        match self {
            ast::Decl::Data(ast::Data { name, .. }) => name,
            ast::Decl::Codata(ast::Codata { name, .. }) => name,
            ast::Decl::Def(ast::Def { name, .. }) => name,
            ast::Decl::Codef(ast::Codef { name, .. }) => name,
            ast::Decl::Ctor(ast::Ctor { name, .. }) => name,
            ast::Decl::Dtor(ast::Dtor { name, .. }) => name,
        }
    }
}

impl Named for cst::Param {
    fn name(&self) -> &Ident {
        &self.name
    }
}

impl Named for ast::Param {
    fn name(&self) -> &Ident {
        &self.name
    }
}

impl Named for cst::EqnParam {
    fn name(&self) -> &Ident {
        &self.name
    }
}

impl Named for ast::EqnParam {
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
