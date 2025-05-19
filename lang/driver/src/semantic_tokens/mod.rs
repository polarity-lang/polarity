use std::sync::Arc;
pub mod types;

use ast::{Codata, Codef, Data, Decl, Def, Let, Module};
use lsp_types::SemanticToken;
use types::{convert_sem_tokens, SemToken};

pub fn compute_semantic_tokens(module: Arc<Module>) -> Vec<SemanticToken> {
    let mut tokens: Vec<SemToken> = Vec::new();
    module.semantic_tokens(&mut tokens);
    convert_sem_tokens(tokens)
}

trait SemTokens {
    fn semantic_tokens(&self, tokens: &mut Vec<SemToken>);
}

impl<T: SemTokens> SemTokens for Vec<T> {
    fn semantic_tokens(&self, tokens: &mut Vec<SemToken>) {
        for x in self {
            x.semantic_tokens(tokens);
        }
    }
}

impl SemTokens for Module {
    fn semantic_tokens(&self, tokens: &mut Vec<SemToken>) {
        let Module { decls, .. } = self;
        decls.semantic_tokens(tokens);
    }
}

impl SemTokens for Decl {
    fn semantic_tokens(&self, tokens: &mut Vec<SemToken>) {
        match self {
            Decl::Data(data) => data.semantic_tokens(tokens),
            Decl::Codata(codata) => codata.semantic_tokens(tokens),
            Decl::Def(def) => def.semantic_tokens(tokens),
            Decl::Codef(codef) => codef.semantic_tokens(tokens),
            Decl::Let(tl_let) => tl_let.semantic_tokens(tokens),
            Decl::Infix(_infix) => {}
        }
    }
}

impl SemTokens for Data {
    fn semantic_tokens(&self, tokens: &mut Vec<SemToken>) {
        let Data { name, .. } = self;
        let st: SemToken = SemToken {
            span: name.span.unwrap(),
            typ: types::TokenType::DataType
        };
        tokens.push(st);
    }
}

impl SemTokens for Codata {
    fn semantic_tokens(&self, _tokens: &mut Vec<SemToken>) {}
}

impl SemTokens for Def {
    fn semantic_tokens(&self, _tokens: &mut Vec<SemToken>) {}
}

impl SemTokens for Codef {
    fn semantic_tokens(&self, _tokens: &mut Vec<SemToken>) {}
}

impl SemTokens for Let {
    fn semantic_tokens(&self, _tokens: &mut Vec<SemToken>) {}
}
