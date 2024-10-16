use ast::{Codef, Ctor, Decl, Def, Dtor, Ident, Let, Named};
use url::Url;

use crate::Database;

pub fn lookup_decl<'a>(db: &'a Database, name: &Ident) -> Option<(Url, &'a Decl)> {
    for uri in db.ust.keys() {
        match db.ust.get_unless_stale(uri) {
            Some(Ok(module)) => {
                if let Some(decl) = module.decls.iter().find(|decl| decl.name() == name) {
                    return Some((uri.clone(), decl));
                }
                continue;
            }
            _ => continue,
        }
    }
    None
}

pub fn lookup_ctor<'a>(_db: &'a Database, _name: &Ident) -> Option<(Url, &'a Ctor)> {
    None
}

pub fn lookup_codef<'a>(_db: &'a Database, _name: &Ident) -> Option<(Url, &'a Codef)> {
    None
}

pub fn lookup_let<'a>(_db: &'a Database, _name: &Ident) -> Option<(Url, &'a Let)> {
    None
}

pub fn lookup_dtor<'a>(_db: &'a Database, _name: &Ident) -> Option<(Url, &'a Dtor)> {
    None
}

pub fn lookup_def<'a>(_db: &'a Database, _name: &Ident) -> Option<(Url, &'a Def)> {
    None
}
