use std::fmt::{self, Display};

use polarity_lang_printer::{Alloc, Builder, DocAllocator, Precedence, Print, PrintCfg};

#[derive(Debug, Clone, PartialEq)]
pub struct Ident {
    pub name: String,
    pub id: Option<usize>,
}

impl Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Ident { name, id } = self;

        match id {
            Some(id) => write!(f, "{name}{id}"),
            None => write!(f, "{name}"),
        }
    }
}

impl From<String> for Ident {
    fn from(s: String) -> Self {
        // split trailing digits
        let n_digits = s.chars().rev().take_while(char::is_ascii_digit).count();
        let (s, digits) = s.split_at(s.len() - n_digits);

        Self { name: s.to_string(), id: digits.parse().ok() }
    }
}

impl Print for Ident {
    fn print_prec<'a>(
        &'a self,
        _cfg: &PrintCfg,
        alloc: &'a Alloc<'a>,
        _prec: Precedence,
    ) -> Builder<'a> {
        alloc.text(self.to_string())
    }
}
