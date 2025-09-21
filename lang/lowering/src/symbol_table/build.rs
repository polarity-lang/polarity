use decls::*;
use miette_util::ToMiette;
use miette_util::codespan::Span;
use parser::cst::*;

use crate::{LoweringError, LoweringResult};

use super::{DeclMeta, ModuleSymbolTable};

pub fn build_symbol_table(module: &Module) -> LoweringResult<ModuleSymbolTable> {
    let mut symbol_table = ModuleSymbolTable::default();

    let Module { decls, .. } = module;

    for decl in decls {
        decl.build(&mut symbol_table)?;
    }

    Ok(symbol_table)
}

/// Checks whether the identifier is reserved or already defined.
fn check_name(symbol_table: &mut ModuleSymbolTable, name: &Ident, span: &Span) -> LoweringResult {
    if name.id == "Type" {
        return Err(LoweringError::TypeUnivIdentifier { span: span.to_miette() }.into());
    }
    if symbol_table.idents.contains_key(name) {
        return Err(LoweringError::AlreadyDefined {
            name: name.to_owned(),
            span: span.to_miette(),
        }
        .into());
    }
    Ok(())
}

trait BuildSymbolTable {
    fn build(&self, symbol_table: &mut ModuleSymbolTable) -> LoweringResult;
}

impl BuildSymbolTable for Decl {
    fn build(&self, symbol_table: &mut ModuleSymbolTable) -> LoweringResult {
        match self {
            Decl::Data(data) => data.build(symbol_table),
            Decl::Codata(codata) => codata.build(symbol_table),
            Decl::Def(def) => def.build(symbol_table),
            Decl::Codef(codef) => codef.build(symbol_table),
            Decl::Let(tl_let) => tl_let.build(symbol_table),
            Decl::Infix(infix) => infix.build(symbol_table),
            Decl::Note(note) => note.build(symbol_table),
        }
    }
}

impl BuildSymbolTable for Data {
    fn build(&self, symbol_table: &mut ModuleSymbolTable) -> LoweringResult {
        let Data { span, name, params, ctors, .. } = self;

        check_name(symbol_table, name, span)?;

        let meta = DeclMeta::Data { params: params.clone() };
        symbol_table.idents.insert(name.clone(), meta);

        for ctor in ctors {
            ctor.build(symbol_table)?;
        }
        Ok(())
    }
}

impl BuildSymbolTable for Ctor {
    fn build(&self, symbol_table: &mut ModuleSymbolTable) -> LoweringResult {
        let Ctor { span, name, params, .. } = self;
        check_name(symbol_table, name, span)?;

        let meta = DeclMeta::Ctor { params: params.clone() };
        symbol_table.idents.insert(name.clone(), meta);

        Ok(())
    }
}

impl BuildSymbolTable for Codata {
    fn build(&self, symbol_table: &mut ModuleSymbolTable) -> LoweringResult {
        let Codata { span, name, params, dtors, .. } = self;
        check_name(symbol_table, name, span)?;

        let meta = DeclMeta::Codata { params: params.clone() };
        symbol_table.idents.insert(name.clone(), meta);

        for dtor in dtors {
            dtor.build(symbol_table)?;
        }
        Ok(())
    }
}

impl BuildSymbolTable for Dtor {
    fn build(&self, symbol_table: &mut ModuleSymbolTable) -> LoweringResult {
        let Dtor { span, name, params, .. } = self;
        check_name(symbol_table, name, span)?;

        let meta = DeclMeta::Dtor { params: params.clone() };
        symbol_table.idents.insert(name.clone(), meta);

        Ok(())
    }
}

impl BuildSymbolTable for Def {
    fn build(&self, symbol_table: &mut ModuleSymbolTable) -> LoweringResult {
        let Def { span, name, params, .. } = self;
        check_name(symbol_table, name, span)?;

        let meta = DeclMeta::Def { params: params.clone() };
        symbol_table.idents.insert(name.clone(), meta);

        Ok(())
    }
}

impl BuildSymbolTable for Codef {
    fn build(&self, symbol_table: &mut ModuleSymbolTable) -> LoweringResult {
        let Codef { span, name, params, .. } = self;
        check_name(symbol_table, name, span)?;

        let meta = DeclMeta::Codef { params: params.clone() };
        symbol_table.idents.insert(name.clone(), meta);

        Ok(())
    }
}

impl BuildSymbolTable for Let {
    fn build(&self, symbol_table: &mut ModuleSymbolTable) -> LoweringResult {
        let Let { span, name, params, .. } = self;
        check_name(symbol_table, name, span)?;

        let meta = DeclMeta::Let { params: params.clone() };
        symbol_table.idents.insert(name.clone(), meta);

        Ok(())
    }
}

impl BuildSymbolTable for Infix {
    fn build(&self, symbol_table: &mut ModuleSymbolTable) -> LoweringResult {
        let Infix { span, doc: _, attr: _, pattern, rhs } = self;

        match pattern.rhs.as_slice() {
            [(operator, _)] => {
                if symbol_table.infix_ops.contains_key(operator) {
                    return Err(LoweringError::OperatorAlreadyDefined {
                        operator: operator.id.to_owned(),
                        span: span.to_miette(),
                    }
                    .into());
                }
                symbol_table.infix_ops.insert(operator.clone(), rhs.name.clone());
            }
            _ => {
                let err = LoweringError::InvalidInfixDeclaration {
                    message: "More than one operator on left hand side".to_string(),
                    span: span.to_miette(),
                };
                return Err(Box::new(err));
            }
        }

        Ok(())
    }
}

impl BuildSymbolTable for Note {
    fn build(&self, symbol_table: &mut ModuleSymbolTable) -> LoweringResult {
        let Note { name, span, .. } = self;

        check_name(symbol_table, name, span)?;
        symbol_table.idents.insert(name.clone(), DeclMeta::Note);

        Ok(())
    }
}
