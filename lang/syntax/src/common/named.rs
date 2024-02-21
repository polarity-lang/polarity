use crate::generic;

pub type Ident = String;

use lazy_static::lazy_static;

pub trait Named {
    fn name(&self) -> &Ident;
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
