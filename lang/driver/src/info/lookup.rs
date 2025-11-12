use polarity_lang_ast::{Codef, Ctor, Decl, Def, Dtor, Extern, IdBound, Let};
use url::Url;

use crate::Database;

pub fn lookup_decl<'a>(db: &'a Database, name: &IdBound) -> Option<(Url, &'a Decl)> {
    let module = db.ust.get_unless_stale(&name.uri)?.as_ref().ok()?;
    let decl = module.decls.iter().find(|decl| match decl.ident() {
        None => false,
        Some(id) => id == name,
    })?;
    Some((name.uri.clone(), decl))
}

pub fn lookup_ctor<'a>(db: &'a Database, name: &IdBound) -> Option<(Url, &'a Ctor)> {
    let module = db.ust.get_unless_stale(&name.uri)?.as_ref().ok()?;
    let ctor = module.decls.iter().find_map(|decl| match decl {
        Decl::Data(data) => data.ctors.iter().find(|ctor| &ctor.name == name),
        _ => None,
    })?;
    Some((name.uri.clone(), ctor))
}

pub fn lookup_codef<'a>(db: &'a Database, name: &IdBound) -> Option<(Url, &'a Codef)> {
    let module = db.ust.get_unless_stale(&name.uri)?.as_ref().ok()?;
    let codef = module.decls.iter().find_map(|decl| match decl {
        Decl::Codef(codef) if codef.name == *name => Some(codef),
        _ => None,
    })?;
    Some((name.uri.clone(), codef))
}

pub fn lookup_let<'a>(db: &'a Database, name: &IdBound) -> Option<(Url, &'a Let)> {
    let module = db.ust.get_unless_stale(&name.uri)?.as_ref().ok()?;
    let tl_let = module.decls.iter().find_map(|decl| match decl {
        Decl::Let(tl_let) if tl_let.name == *name => Some(tl_let),
        _ => None,
    })?;
    Some((name.uri.clone(), tl_let))
}

pub fn lookup_extern<'a>(db: &'a Database, name: &IdBound) -> Option<(Url, &'a Extern)> {
    let module = db.ust.get_unless_stale(&name.uri)?.as_ref().ok()?;
    let extern_decl = module.decls.iter().find_map(|decl| match decl {
        Decl::Extern(extern_decl) if extern_decl.name == *name => Some(extern_decl),
        _ => None,
    })?;
    Some((name.uri.clone(), extern_decl))
}

pub fn lookup_dtor<'a>(db: &'a Database, name: &IdBound) -> Option<(Url, &'a Dtor)> {
    let module = db.ust.get_unless_stale(&name.uri)?.as_ref().ok()?;
    let dtor = module.decls.iter().find_map(|decl| match decl {
        Decl::Codata(codata) => codata.dtors.iter().find(|dtor| &dtor.name == name),
        _ => None,
    })?;
    Some((name.uri.clone(), dtor))
}

pub fn lookup_def<'a>(db: &'a Database, name: &IdBound) -> Option<(Url, &'a Def)> {
    let module = db.ust.get_unless_stale(&name.uri)?.as_ref().ok()?;
    let def = module.decls.iter().find_map(|decl| match decl {
        Decl::Def(def) if def.name == *name => Some(def),
        _ => None,
    })?;
    Some((name.uri.clone(), def))
}
