use crate::generic;
use parser::cst::Ident;
use parser::cst::{self, BindingSite};

use lazy_static::lazy_static;

pub trait Named {
    fn name(&self) -> &Ident;
}

impl Named for cst::Item {
    fn name(&self) -> &Ident {
        match self {
            cst::Item::Data(cst::Data { name, .. }) => name,
            cst::Item::Codata(cst::Codata { name, .. }) => name,
            cst::Item::Def(cst::Def { name, .. }) => name,
            cst::Item::Codef(cst::Codef { name, .. }) => name,
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

lazy_static! {
    static ref WILDCARD: String = "_".to_owned();
}

impl Named for cst::Param {
    fn name(&self) -> &Ident {
        self.name.name()
    }
}

impl Named for cst::ParamInst {
    fn name(&self) -> &Ident {
        self.name.name()
    }
}

impl Named for BindingSite {
    fn name(&self) -> &Ident {
        match &self {
            BindingSite::Var { name } => name.name(),
            BindingSite::Wildcard => &WILDCARD,
        }
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
