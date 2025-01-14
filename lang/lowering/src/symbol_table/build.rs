use ast::HashMap;
use codespan::Span;
use decls::*;
use miette_util::ToMiette;
use parser::cst::*;

use crate::LoweringError;

use super::{DeclMeta, ModuleSymbolTable};

pub fn build_symbol_table(module: &Module) -> Result<ModuleSymbolTable, LoweringError> {
    let mut symbol_table = HashMap::default();

    let Module { decls, .. } = module;

    for decl in decls {
        decl.build(&mut symbol_table)?;
    }

    Ok(symbol_table)
}

fn check_ident(ident: &Ident, span: &Span) -> Result<(), LoweringError> {
    if ident.id == "Type" {
        return Err(LoweringError::TypeUnivIdentifier { span: span.to_miette() });
    }
    Ok(())
}

trait BuildSymbolTable {
    fn build(&self, symbol_table: &mut ModuleSymbolTable) -> Result<(), LoweringError>;
}

impl BuildSymbolTable for Decl {
    fn build(&self, symbol_table: &mut ModuleSymbolTable) -> Result<(), LoweringError> {
        match self {
            Decl::Data(data) => data.build(symbol_table),
            Decl::Codata(codata) => codata.build(symbol_table),
            Decl::Def(def) => def.build(symbol_table),
            Decl::Codef(codef) => codef.build(symbol_table),
            Decl::Let(tl_let) => tl_let.build(symbol_table),
        }
    }
}

impl BuildSymbolTable for Data {
    fn build(&self, symbol_table: &mut ModuleSymbolTable) -> Result<(), LoweringError> {
        let Data { span, name, params, ctors, .. } = self;
        check_ident(name, span)?;
        match symbol_table.get(name) {
            Some(_) => {
                return Err(LoweringError::AlreadyDefined {
                    name: name.to_owned(),
                    span: span.to_miette(),
                });
            }
            None => {
                let meta = DeclMeta::Data { params: params.clone() };
                symbol_table.insert(name.clone(), meta);
            }
        }
        for ctor in ctors {
            ctor.build(symbol_table)?;
        }
        Ok(())
    }
}

impl BuildSymbolTable for Ctor {
    fn build(&self, symbol_table: &mut ModuleSymbolTable) -> Result<(), LoweringError> {
        let Ctor { span, name, params, .. } = self;
        check_ident(name, span)?;
        match symbol_table.get(name) {
            Some(_) => {
                return Err(LoweringError::AlreadyDefined {
                    name: name.to_owned(),
                    span: span.to_miette(),
                });
            }
            None => {
                let meta = DeclMeta::Ctor { params: params.clone() };
                symbol_table.insert(name.clone(), meta);
            }
        }
        Ok(())
    }
}

impl BuildSymbolTable for Codata {
    fn build(&self, symbol_table: &mut ModuleSymbolTable) -> Result<(), LoweringError> {
        let Codata { span, name, params, dtors, .. } = self;
        check_ident(name, span)?;
        match symbol_table.get(name) {
            Some(_) => {
                return Err(LoweringError::AlreadyDefined {
                    name: name.to_owned(),
                    span: span.to_miette(),
                });
            }
            None => {
                let meta = DeclMeta::Codata { params: params.clone() };
                symbol_table.insert(name.clone(), meta);
            }
        }
        for dtor in dtors {
            dtor.build(symbol_table)?;
        }
        Ok(())
    }
}

impl BuildSymbolTable for Dtor {
    fn build(&self, symbol_table: &mut ModuleSymbolTable) -> Result<(), LoweringError> {
        let Dtor { span, name, params, .. } = self;
        check_ident(name, span)?;
        match symbol_table.get(name) {
            Some(_) => {
                return Err(LoweringError::AlreadyDefined {
                    name: name.to_owned(),
                    span: span.to_miette(),
                });
            }
            None => {
                let meta = DeclMeta::Dtor { params: params.clone() };
                symbol_table.insert(name.clone(), meta);
            }
        }
        Ok(())
    }
}

impl BuildSymbolTable for Def {
    fn build(&self, symbol_table: &mut ModuleSymbolTable) -> Result<(), LoweringError> {
        let Def { span, name, params, .. } = self;
        check_ident(name, span)?;

        match symbol_table.get(name) {
            Some(_) => {
                return Err(LoweringError::AlreadyDefined {
                    name: name.to_owned(),
                    span: span.to_miette(),
                });
            }
            None => {
                let meta = DeclMeta::Def { params: params.clone() };
                symbol_table.insert(name.clone(), meta);
            }
        }
        Ok(())
    }
}

impl BuildSymbolTable for Codef {
    fn build(&self, symbol_table: &mut ModuleSymbolTable) -> Result<(), LoweringError> {
        let Codef { span, name, params, .. } = self;
        check_ident(name, span)?;

        match symbol_table.get(name) {
            Some(_) => {
                return Err(LoweringError::AlreadyDefined {
                    name: name.to_owned(),
                    span: span.to_miette(),
                });
            }
            None => {
                let meta = DeclMeta::Codef { params: params.clone() };
                symbol_table.insert(name.clone(), meta);
            }
        }
        Ok(())
    }
}

impl BuildSymbolTable for Let {
    fn build(&self, symbol_table: &mut ModuleSymbolTable) -> Result<(), LoweringError> {
        let Let { span, name, params, .. } = self;
        check_ident(name, span)?;
        match symbol_table.get(name) {
            Some(_) => {
                return Err(LoweringError::AlreadyDefined {
                    name: name.to_owned(),
                    span: span.to_miette(),
                });
            }
            None => {
                let meta = DeclMeta::Let { params: params.clone() };
                symbol_table.insert(name.clone(), meta);
            }
        }
        Ok(())
    }
}
