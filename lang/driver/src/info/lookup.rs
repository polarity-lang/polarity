use ast::{Codef, Ctor, Decl, Def, Dtor, Ident, Let};
use url::Url;

use crate::Database;

pub fn lookup_decl<'a>(_db: &'a Database, _name: &Ident) -> Option<(Url, &'a Decl)> {
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
