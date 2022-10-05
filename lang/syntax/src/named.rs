use super::ast;
use super::common::*;
use super::cst;

pub trait Named {
    fn name(&self) -> &Ident;
}

impl Named for cst::Decl {
    fn name(&self) -> &Ident {
        match self {
            cst::Decl::Data(cst::Data { name, .. }) => name,
            cst::Decl::Codata(cst::Codata { name, .. }) => name,
            cst::Decl::Def(cst::Def { name, .. }) => name,
            cst::Decl::Codef(cst::Codef { name, .. }) => name,
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
