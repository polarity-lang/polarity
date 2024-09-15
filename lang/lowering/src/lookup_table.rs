use std::fmt;

use ast::HashMap;
use codespan::Span;
use decls::*;
use ident::Ident;
use miette_util::ToMiette;
use parser::cst::*;

use crate::LoweringError;

#[derive(Default)]
pub struct LookupTable {
    map: HashMap<Ident, DeclMeta>,
}

impl LookupTable {
    pub fn lookup(&self, name: &Ident) -> Option<&DeclMeta> {
        self.map.get(name)
    }

    fn add(&mut self, name: &Ident, span: &Span, decl_meta: DeclMeta) -> Result<(), LoweringError> {
        if self.map.contains_key(name) {
            return Err(LoweringError::AlreadyDefined {
                name: name.to_owned(),
                span: Some(span.to_miette()),
            });
        }
        self.map.insert(name.clone(), decl_meta);
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum DeclMeta {
    Data { params: Telescope },
    Codata { params: Telescope },
    Def { params: Telescope },
    Codef { params: Telescope },
    Ctor { params: Telescope },
    Dtor { params: Telescope },
    Let { params: Telescope },
}

impl DeclMeta {
    pub fn kind(&self) -> DeclKind {
        match self {
            DeclMeta::Data { .. } => DeclKind::Data,
            DeclMeta::Codata { .. } => DeclKind::Codata,
            DeclMeta::Def { .. } => DeclKind::Def,
            DeclMeta::Codef { .. } => DeclKind::Codef,
            DeclMeta::Ctor { .. } => DeclKind::Ctor,
            DeclMeta::Dtor { .. } => DeclKind::Dtor,
            DeclMeta::Let { .. } => DeclKind::Let,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum DeclKind {
    Data,
    Codata,
    Def,
    Codef,
    Ctor,
    Dtor,
    Let,
}

impl fmt::Display for DeclKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeclKind::Data => write!(f, "data type"),
            DeclKind::Codata => write!(f, "codata type"),
            DeclKind::Def => write!(f, "definition"),
            DeclKind::Codef => write!(f, "codefinition"),
            DeclKind::Ctor => write!(f, "constructor"),
            DeclKind::Dtor => write!(f, "destructor"),
            DeclKind::Let => write!(f, "toplevel let"),
        }
    }
}

pub fn build_lookup_table(module: &Module) -> Result<LookupTable, LoweringError> {
    let mut lookup_table = LookupTable { map: HashMap::default() };

    let Module { decls, .. } = module;

    for decl in decls {
        match decl {
            Decl::Data(data) => build_data(&mut lookup_table, data)?,
            Decl::Codata(codata) => build_codata(&mut lookup_table, codata)?,
            Decl::Def(def) => build_def(&mut lookup_table, def)?,
            Decl::Codef(codef) => build_codef(&mut lookup_table, codef)?,
            Decl::Let(tl_let) => build_let(&mut lookup_table, tl_let)?,
        }
    }

    Ok(lookup_table)
}

fn build_data(lookup_table: &mut LookupTable, data: &Data) -> Result<(), LoweringError> {
    let Data { span, name, params, ctors, .. } = data;
    lookup_table.add(name, span, DeclMeta::Data { params: params.clone() })?;
    for ctor in ctors {
        build_ctor(lookup_table, ctor)?;
    }
    Ok(())
}

fn build_ctor(lookup_table: &mut LookupTable, ctor: &Ctor) -> Result<(), LoweringError> {
    let Ctor { span, name, params, .. } = ctor;
    lookup_table.add(name, span, DeclMeta::Ctor { params: params.clone() })?;
    Ok(())
}

fn build_codata(lookup_table: &mut LookupTable, codata: &Codata) -> Result<(), LoweringError> {
    let Codata { span, name, params, dtors, .. } = codata;
    lookup_table.add(name, span, DeclMeta::Codata { params: params.clone() })?;
    for dtor in dtors {
        build_dtor(lookup_table, dtor)?;
    }
    Ok(())
}

fn build_dtor(lookup_table: &mut LookupTable, dtor: &Dtor) -> Result<(), LoweringError> {
    let Dtor { span, name, params, .. } = dtor;
    lookup_table.add(name, span, DeclMeta::Dtor { params: params.clone() })?;
    Ok(())
}

fn build_def(lookup_table: &mut LookupTable, def: &Def) -> Result<(), LoweringError> {
    let Def { span, name, params, .. } = def;
    lookup_table.add(name, span, DeclMeta::Def { params: params.clone() })?;
    Ok(())
}

fn build_codef(lookup_table: &mut LookupTable, codef: &Codef) -> Result<(), LoweringError> {
    let Codef { span, name, params, .. } = codef;
    lookup_table.add(name, span, DeclMeta::Codef { params: params.clone() })?;
    Ok(())
}

fn build_let(lookup_table: &mut LookupTable, tl_let: &Let) -> Result<(), LoweringError> {
    let Let { span, name, params, .. } = tl_let;
    lookup_table.add(name, span, DeclMeta::Let { params: params.clone() })?;
    Ok(())
}
