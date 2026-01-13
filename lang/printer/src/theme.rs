use std::fmt::Display;

use pretty::DocAllocator;

use super::types::*;

pub trait ThemeExt<'a> {
    fn keyword(&'a self, text: impl Display) -> Builder<'a>;
    fn ctor(&'a self, text: impl Display) -> Builder<'a>;
    fn dtor(&'a self, text: impl Display) -> Builder<'a>;
    fn typ(&'a self, text: impl Display) -> Builder<'a>;
    fn comment(&'a self, text: impl Display) -> Builder<'a>;
}

impl<'a> ThemeExt<'a> for Alloc<'a> {
    fn keyword(&'a self, text: impl Display) -> Builder<'a> {
        self.text(text.to_string()).annotate(Anno::Keyword)
    }

    fn ctor(&'a self, text: impl Display) -> Builder<'a> {
        self.text(text.to_string()).annotate(Anno::Ctor)
    }

    fn dtor(&'a self, text: impl Display) -> Builder<'a> {
        self.text(text.to_string()).annotate(Anno::Dtor)
    }

    fn typ(&'a self, text: impl Display) -> Builder<'a> {
        self.text(text.to_string()).annotate(Anno::Type)
    }

    fn comment(&'a self, text: impl Display) -> Builder<'a> {
        self.text(text.to_string()).annotate(Anno::Comment)
    }
}
