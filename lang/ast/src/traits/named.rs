use crate::Ident;

/// Trait for syntactic items which have a unique `name`.
pub trait Named {
    fn name(&self) -> &Ident;
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
