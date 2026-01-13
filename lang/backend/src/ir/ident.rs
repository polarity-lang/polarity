use std::fmt::{self, Display};

use polarity_lang_printer::{Alloc, Builder, DocAllocator, Precedence, Print, PrintCfg};

use crate::ir::rename::{Rename, RenameCtx};

#[derive(Debug, Clone)]
pub struct Ident {
    pub name: String,
    pub id: Option<usize>,
}

impl Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.id {
            Some(id) => write!(f, "{}{id}", self.name),
            None => write!(f, "{}", self.name),
        }
    }
}

impl From<String> for Ident {
    fn from(value: String) -> Self {
        Self { name: value, id: None }
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

impl Rename for Ident {
    fn rename(&mut self, ctx: &mut RenameCtx) {
        todo!()
    }
}
