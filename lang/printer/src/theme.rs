use pretty::DocAllocator;
use url::Url;

use super::types::*;

pub trait ThemeExt<'a> {
    fn keyword(&'a self, text: &str) -> Builder<'a>;
    fn ctor(&'a self, text: &str) -> Builder<'a>;
    fn dtor(&'a self, text: &str) -> Builder<'a>;
    fn typ(&'a self, text: &str) -> Builder<'a>;
    fn comment(&'a self, text: &str) -> Builder<'a>;
    fn reference(&'a self, module_uri: Url, text: &str) -> Builder<'a>;
}

impl<'a> ThemeExt<'a> for Alloc<'a> {
    fn keyword(&'a self, text: &str) -> Builder<'a> {
        self.text(text.to_owned()).annotate(Anno::Keyword)
    }

    fn ctor(&'a self, text: &str) -> Builder<'a> {
        self.text(text.to_owned()).annotate(Anno::Ctor)
    }

    fn dtor(&'a self, text: &str) -> Builder<'a> {
        self.text(text.to_owned()).annotate(Anno::Dtor)
    }

    fn typ(&'a self, text: &str) -> Builder<'a> {
        self.text(text.to_owned()).annotate(Anno::Type)
    }

    fn comment(&'a self, text: &str) -> Builder<'a> {
        self.text(text.to_owned()).annotate(Anno::Comment)
    }
    fn reference(&'a self, module_uri: Url, text: &str) -> Builder<'a> {
        self.text(text.to_owned()).annotate(Anno::Reference { module_uri: module_uri.to_owned(), name: text.to_owned() })
    }
}
