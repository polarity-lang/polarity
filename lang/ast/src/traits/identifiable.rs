use crate::IdBind;

/// Trait for syntactic items which have a unique `name`.
pub trait GloballyIdentifiable {
    fn ident(&self) -> &IdBind;
}

impl GloballyIdentifiable for IdBind {
    fn ident(&self) -> &IdBind {
        self
    }
}

impl<'a, T> GloballyIdentifiable for &'a T
where
    T: GloballyIdentifiable,
{
    fn ident(&self) -> &IdBind {
        T::ident(self)
    }
}
