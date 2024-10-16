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

pub fn lookup_ctor<'a>(db: &'a Database, name: &Ident) -> Option<(Url, &'a Ctor)> {
    for uri in db.ust.keys() {
        match db.ust.get_unless_stale(uri) {
            Some(Ok(module)) => {
                if let Some(ctor) = module.decls.iter().find_map(|decl| match decl {
                    Decl::Data(data) => data.ctors.iter().find(|ctor| &ctor.name == name),
                    _ => None,
                }) {
                    return Some((uri.clone(), ctor));
                }
            }
            _ => continue,
        }
    }
    None
}

pub fn lookup_codef<'a>(db: &'a Database, name: &Ident) -> Option<(Url, &'a Codef)> {
    for uri in db.ust.keys() {
        match db.ust.get_unless_stale(uri) {
            Some(Ok(module)) => {
                if let Some(codef) = module.decls.iter().find_map(|decl| match decl {
                    Decl::Codef(codef) if codef.name == *name => Some(codef),
                    _ => None,
                }) {
                    return Some((uri.clone(), codef));
                }
            }
            _ => continue,
        }
    }
    None
}

pub fn lookup_let<'a>(db: &'a Database, name: &Ident) -> Option<(Url, &'a Let)> {
    for uri in db.ust.keys() {
        match db.ust.get_unless_stale(uri) {
            Some(Ok(module)) => {
                if let Some(tl_let) = module.decls.iter().find_map(|decl| match decl {
                    Decl::Let(tl_let) if tl_let.name == *name => Some(tl_let),
                    _ => None,
                }) {
                    return Some((uri.clone(), tl_let));
                }
            }
            _ => continue,
        }
    }
    None
}

pub fn lookup_dtor<'a>(db: &'a Database, name: &Ident) -> Option<(Url, &'a Dtor)> {
    for uri in db.ust.keys() {
        match db.ust.get_unless_stale(uri) {
            Some(Ok(module)) => {
                if let Some(dtor) = module.decls.iter().find_map(|decl| match decl {
                    Decl::Codata(codata) => codata.dtors.iter().find(|dtor| &dtor.name == name),
                    _ => None,
                }) {
                    return Some((uri.clone(), dtor));
                }
            }
            _ => continue,
        }
    }
    None
}

pub fn lookup_def<'a>(db: &'a Database, name: &Ident) -> Option<(Url, &'a Def)> {
    for uri in db.ust.keys() {
        match db.ust.get_unless_stale(uri) {
            Some(Ok(module)) => {
                if let Some(def) = module.decls.iter().find_map(|decl| match decl {
                    Decl::Def(def) if def.name == *name => Some(def),
                    _ => None,
                }) {
                    return Some((uri.clone(), def));
                }
            }
            _ => continue,
        }
    }
    None
}
