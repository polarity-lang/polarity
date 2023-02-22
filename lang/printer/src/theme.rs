use pretty::{
    termcolor::{Color, ColorSpec},
    DocAllocator,
};

use super::types::*;

const KEYWORD: Color = Color::Magenta;
const CTOR: Color = Color::Blue;
const DTOR: Color = Color::Green;
const TYPE: Color = Color::Red;
const COMMENT: Color = Color::Cyan;

pub trait ThemeExt<'a> {
    fn keyword(&'a self, text: &'a str) -> Builder<'a>;
    fn ctor(&'a self, text: &'a str) -> Builder<'a>;
    fn dtor(&'a self, text: &'a str) -> Builder<'a>;
    fn typ(&'a self, text: &'a str) -> Builder<'a>;
    fn comment(&'a self, text: &'a str) -> Builder<'a>;
}

impl<'a> ThemeExt<'a> for Alloc<'a> {
    fn keyword(&'a self, text: &'a str) -> Builder<'a> {
        self.text(text).annotate(KEYWORD.spec())
    }

    fn ctor(&'a self, text: &'a str) -> Builder<'a> {
        self.text(text).annotate(CTOR.spec())
    }

    fn dtor(&'a self, text: &'a str) -> Builder<'a> {
        self.text(text).annotate(DTOR.spec())
    }

    fn typ(&'a self, text: &'a str) -> Builder<'a> {
        self.text(text).annotate(TYPE.spec())
    }

    fn comment(&'a self, text: &'a str) -> Builder<'a> {
        self.text(text).annotate(COMMENT.spec())
    }
}

pub trait ColorExt {
    fn spec(self) -> ColorSpec;
}

impl ColorExt for Color {
    fn spec(self) -> ColorSpec {
        ColorSpec::new().set_fg(Some(self)).clone()
    }
}
