use std::rc::Rc;

use crate::common::*;
use miette::{Diagnostic, SourceSpan};
use miette_util::ToMiette;
use thiserror::Error;

use super::decls::*;
use super::exp::*;
use super::lookup_table;
use super::lookup_table::DeclKind;

impl<P: Phase> Decls<P> {
    pub fn empty() -> Self {
        Self { map: HashMap::default(), lookup_table: Default::default() }
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = Item<'_, P>> {
        self.lookup_table.iter().map(|item| match item {
            lookup_table::Item::Type(type_decl) => match &self.map[&type_decl.name] {
                Decl::Data(data) => Item::Data(data),
                Decl::Codata(codata) => Item::Codata(codata),
                _ => unreachable!(),
            },
            lookup_table::Item::Def(def_decl) => match &self.map[&def_decl.name] {
                Decl::Def(def) => Item::Def(def),
                Decl::Codef(codef) => Item::Codef(codef),
                _ => unreachable!(),
            },
            lookup_table::Item::Let { name } => match &self.map[name] {
                Decl::Let(tl_let) => Item::Let(tl_let),
                _ => unreachable!(),
            },
        })
    }

    pub fn type_decl_for_member(
        &self,
        name: &Ident,
        span: Option<codespan::Span>,
    ) -> Result<Polarity<'_, P>, LookupError> {
        let type_decl = self
            .lookup_table
            .type_decl_for_xtor(name)
            .or_else(|| self.lookup_table.type_decl_for_xdef(name))
            .ok_or_else(|| LookupError::MissingTypeDeclaration {
                name: name.to_owned(),
                span: span.to_miette(),
            })?;
        self.typ(&type_decl.name, None)
    }

    pub fn data_for_ctor(
        &self,
        name: &str,
        span: Option<codespan::Span>,
    ) -> Result<&Data<P>, LookupError> {
        self.ctor(name, span)?;
        let type_decl = self.lookup_table.type_decl_for_xtor(name).ok_or_else(|| {
            LookupError::MissingTypeDeclaration { name: name.to_owned(), span: span.to_miette() }
        })?;
        self.data(&type_decl.name, None)
    }

    pub fn codata_for_dtor(
        &self,
        name: &str,
        span: Option<codespan::Span>,
    ) -> Result<&Codata<P>, LookupError> {
        self.dtor(name, span)?;
        let type_decl = self.lookup_table.type_decl_for_xtor(name).ok_or_else(|| {
            LookupError::MissingTypeDeclaration { name: name.to_owned(), span: span.to_miette() }
        })?;
        self.codata(&type_decl.name, None)
    }

    pub fn xtors_for_type(&self, name: &str) -> Vec<Ident> {
        self.lookup_table.xtors_for_type(name)
    }

    pub fn xdefs_for_type(&self, name: &str) -> Vec<Ident> {
        self.lookup_table.xdefs_for_type(name)
    }

    pub fn typ(
        &self,
        name: &str,
        span: Option<codespan::Span>,
    ) -> Result<Polarity<'_, P>, LookupError> {
        match self.decl(name, span)? {
            Decl::Data(data) => Ok(Polarity::Data(data)),
            Decl::Codata(codata) => Ok(Polarity::Codata(codata)),
            other => Err(LookupError::ExpectedDataCodata {
                name: name.to_owned(),
                actual: other.kind(),
                span: span.to_miette(),
            }),
        }
    }

    pub fn data(&self, name: &str, span: Option<codespan::Span>) -> Result<&Data<P>, LookupError> {
        match self.decl(name, span)? {
            Decl::Data(data) => Ok(data),
            other => Err(LookupError::ExpectedData {
                name: name.to_owned(),
                actual: other.kind(),
                span: span.to_miette(),
            }),
        }
    }

    pub fn codata(
        &self,
        name: &str,
        span: Option<codespan::Span>,
    ) -> Result<&Codata<P>, LookupError> {
        match self.decl(name, span)? {
            Decl::Codata(codata) => Ok(codata),
            other => Err(LookupError::ExpectedCodata {
                name: name.to_owned(),
                actual: other.kind(),
                span: span.to_miette(),
            }),
        }
    }

    pub fn def(&self, name: &str, span: Option<codespan::Span>) -> Result<&Def<P>, LookupError> {
        match self.decl(name, span)? {
            Decl::Def(def) => Ok(def),
            other => Err(LookupError::ExpectedDef {
                name: name.to_owned(),
                actual: other.kind(),
                span: span.to_miette(),
            }),
        }
    }

    pub fn codef(
        &self,
        name: &str,
        span: Option<codespan::Span>,
    ) -> Result<&Codef<P>, LookupError> {
        match self.decl(name, span)? {
            Decl::Codef(codef) => Ok(codef),
            other => Err(LookupError::ExpectedCodef {
                name: name.to_owned(),
                actual: other.kind(),
                span: span.to_miette(),
            }),
        }
    }

    pub fn ctor(&self, name: &str, span: Option<codespan::Span>) -> Result<&Ctor<P>, LookupError> {
        match self.decl(name, span)? {
            Decl::Ctor(ctor) => Ok(ctor),
            other => Err(LookupError::ExpectedCtor {
                name: name.to_owned(),
                actual: other.kind(),
                span: span.to_miette(),
            }),
        }
    }

    pub fn dtor(&self, name: &str, span: Option<codespan::Span>) -> Result<&Dtor<P>, LookupError> {
        match self.decl(name, span)? {
            Decl::Dtor(dtor) => Ok(dtor),
            other => Err(LookupError::ExpectedDtor {
                name: name.to_owned(),
                actual: other.kind(),
                span: span.to_miette(),
            }),
        }
    }

    pub fn ctor_or_codef(
        &self,
        name: &str,
        span: Option<codespan::Span>,
    ) -> Result<Ctor<P>, LookupError> {
        match self.decl(name, span)? {
            Decl::Ctor(ctor) => Ok(ctor.clone()),
            Decl::Codef(codef) => Ok(codef.to_ctor()),
            other => Err(LookupError::ExpectedCtorCodef {
                name: name.to_owned(),
                actual: other.kind(),
                span: span.to_miette(),
            }),
        }
    }

    pub fn dtor_or_def(
        &self,
        name: &str,
        span: Option<codespan::Span>,
    ) -> Result<Dtor<P>, LookupError> {
        match self.decl(name, span)? {
            Decl::Dtor(dtor) => Ok(dtor.clone()),
            Decl::Def(def) => Ok(def.to_dtor()),
            other => Err(LookupError::ExpectedDtorDef {
                name: name.to_owned(),
                actual: other.kind(),
                span: span.to_miette(),
            }),
        }
    }

    fn decl(&self, name: &str, span: Option<codespan::Span>) -> Result<&Decl<P>, LookupError> {
        self.map.get(name).ok_or_else(|| LookupError::UndefinedDeclaration {
            name: name.to_owned(),
            span: span.to_miette(),
        })
    }
}

#[derive(Debug, Clone)]
pub enum Item<'a, P: Phase> {
    Data(&'a Data<P>),
    Codata(&'a Codata<P>),
    Def(&'a Def<P>),
    Codef(&'a Codef<P>),
    Let(&'a Let<P>),
}

impl<'a, P: Phase> Item<'a, P> {
    pub fn attributes(&self) -> &Attribute {
        match self {
            Item::Data(data) => &data.attr,
            Item::Codata(codata) => &codata.attr,
            Item::Def(def) => &def.attr,
            Item::Codef(codef) => &codef.attr,
            Item::Let(tl_let) => &tl_let.attr,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Polarity<'a, P: Phase> {
    Data(&'a Data<P>),
    Codata(&'a Codata<P>),
}

impl<'a, P: Phase> Polarity<'a, P> {
    pub fn name(&self) -> &Ident {
        match self {
            Polarity::Data(data) => &data.name,
            Polarity::Codata(codata) => &codata.name,
        }
    }

    pub fn typ(&self) -> Rc<TypAbs<P>> {
        match self {
            Polarity::Data(data) => data.typ.clone(),
            Polarity::Codata(codata) => codata.typ.clone(),
        }
    }
}

#[derive(Error, Diagnostic, Debug)]
pub enum LookupError {
    #[error("Undefined top-level declaration {name}")]
    UndefinedDeclaration {
        name: String,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Expected {name} to be a data type or codata type, but it is a {actual}")]
    ExpectedDataCodata {
        name: String,
        actual: DeclKind,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Expected {name} to be a data type, but it is a {actual}")]
    ExpectedData {
        name: String,
        actual: DeclKind,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Expected {name} to be a codata type, but it is a {actual}")]
    ExpectedCodata {
        name: String,
        actual: DeclKind,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Expected {name} to be a constructor or codefinition, but it is a {actual}")]
    ExpectedCtorCodef {
        name: String,
        actual: DeclKind,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Expected {name} to be a constructor, but it is a {actual}")]
    ExpectedCtor {
        name: String,
        actual: DeclKind,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Expected {name} to be a codefinition, but it is a {actual}")]
    ExpectedCodef {
        name: String,
        actual: DeclKind,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Expected {name} to be a destructor or definition, but it is a {actual}")]
    ExpectedDtorDef {
        name: String,
        actual: DeclKind,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Expected {name} to be a destructor, but it is a {actual}")]
    ExpectedDtor {
        name: String,
        actual: DeclKind,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Expected {name} to be a definition, but it is a {actual}")]
    ExpectedDef {
        name: String,
        actual: DeclKind,
        #[label]
        span: Option<SourceSpan>,
    },
    #[error("Missing type declaration for {name}")]
    MissingTypeDeclaration {
        name: String,
        #[label]
        span: Option<SourceSpan>,
    },
}
