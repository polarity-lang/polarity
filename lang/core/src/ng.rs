//! Name generator for (co)match labels

use data::HashMap;
use syntax::ast::Type;
use syntax::common::*;
use syntax::ust::Prg;

#[derive(Debug, Default)]
pub struct NameGen {
    map: HashMap<Ident, usize>,
}

impl NameGen {
    pub fn fresh_label(&mut self, type_name: &str, prg: &Prg) -> Ident {
        let i = self.map.entry(type_name.to_owned()).or_default();
        loop {
            let name = match prg.decls.typ(type_name) {
                Type::Data(_) => {
                    let lowered = type_name.to_lowercase();
                    format!("d_{lowered}{i}")
                }
                Type::Codata(_) => format!("Mk{type_name}{i}"),
            };
            *i += 1;
            if !prg.decls.map.contains_key(&name) {
                return name;
            }
        }
    }
}
