use std::sync::Arc;

use ast::{Codata, Codef, Data, Decl, Def, Let, Module};
use lsp_types::SemanticToken;

pub fn compute_semantic_tokens(module: Arc<Module>) -> Vec<SemanticToken> {
    let mut tokens: Vec<SemanticToken> = Vec::new();
    module.semantic_tokens(&mut tokens);
    tokens
}

trait SemTokens {
    fn semantic_tokens(&self, tokens: &mut Vec<SemanticToken>);
}

impl<T: SemTokens> SemTokens for Vec<T> {
    fn semantic_tokens(&self, tokens: &mut Vec<SemanticToken>) {
        for x in self {
            x.semantic_tokens(tokens);
        }
    }
}

impl SemTokens for Module {
    fn semantic_tokens(&self, tokens: &mut Vec<SemanticToken>) {
        let Module { decls, .. } = self;
        decls.semantic_tokens(tokens);
    }
}

impl SemTokens for Decl {
    fn semantic_tokens(&self, tokens: &mut Vec<SemanticToken>) {
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
    fn semantic_tokens(&self, _tokens: &mut Vec<SemanticToken>) {}
}

impl SemTokens for Codata {
    fn semantic_tokens(&self, _tokens: &mut Vec<SemanticToken>) {}
}

impl SemTokens for Def {
    fn semantic_tokens(&self, _tokens: &mut Vec<SemanticToken>) {}
}

impl SemTokens for Codef {
    fn semantic_tokens(&self, _tokens: &mut Vec<SemanticToken>) {}
}

impl SemTokens for Let {
    fn semantic_tokens(&self, _tokens: &mut Vec<SemanticToken>) {}
}
