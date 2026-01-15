use std::fmt::{self, Display};

use polarity_lang_printer::{Alloc, Builder, DocAllocator, Precedence, Print, PrintCfg};

use crate::ir::rename::{Rename, RenameCtx, rename_to_valid_identifier};

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

impl Rename for Ident {
    fn rename(&mut self, ctx: &mut RenameCtx) {
        let original = self.clone();

        rename_to_valid_identifier(&mut self.name, ctx.backend);

        // if there's an exact match of the original name, reuse the binding.
        if let Some(same) = ctx.binders.iter().find(|other| original == other.0) {
            *self = same.1.clone();
            return;
        }

        let occupied_ids: Vec<_> = ctx
            .binders
            .iter()
            .filter(|other| *self.name == other.1.name)
            .map(|other| other.1.id)
            .collect();

        if occupied_ids.contains(&self.id) {
            self.id = smallest_non_occupied_id(&occupied_ids);
        }

        ctx.binders.push((original, self.clone()));
    }
}

fn smallest_non_occupied_id(occupied_ids: &[Option<usize>]) -> Option<usize> {
    if !occupied_ids.contains(&None) {
        return None;
    }

    for id in 0.. {
        if !occupied_ids.contains(&Some(id)) {
            return Some(id);
        }
    }

    unreachable!()
}
