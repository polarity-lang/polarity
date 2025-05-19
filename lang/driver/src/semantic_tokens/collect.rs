//! Traversing an AST to collect the semantic tokens.

use ast::{Codata, Codef, Data, Decl, Def, Let, Module};

use super::types::SemToken;
pub trait SemTokens {
    fn collect_tokens(&self, tokens: &mut Vec<SemToken>);
}

impl<T: SemTokens> SemTokens for Vec<T> {
    fn collect_tokens(&self, tokens: &mut Vec<SemToken>) {
        for x in self {
            x.collect_tokens(tokens);
        }
    }
}

impl SemTokens for Module {
    fn collect_tokens(&self, tokens: &mut Vec<SemToken>) {
        let Module { decls, .. } = self;
        decls.collect_tokens(tokens);
    }
}

impl SemTokens for Decl {
    fn collect_tokens(&self, tokens: &mut Vec<SemToken>) {
        match self {
            Decl::Data(data) => data.collect_tokens(tokens),
            Decl::Codata(codata) => codata.collect_tokens(tokens),
            Decl::Def(def) => def.collect_tokens(tokens),
            Decl::Codef(codef) => codef.collect_tokens(tokens),
            Decl::Let(tl_let) => tl_let.collect_tokens(tokens),
            Decl::Infix(_infix) => {}
        }
    }
}

impl SemTokens for Data {
    fn collect_tokens(&self, tokens: &mut Vec<SemToken>) {
        let Data { name, .. } = self;
        let st: SemToken =
            SemToken { span: name.span.unwrap(), typ: super::types::TokenType::DataType };
        tokens.push(st);
    }
}

impl SemTokens for Codata {
    fn collect_tokens(&self, _tokens: &mut Vec<SemToken>) {}
}

impl SemTokens for Def {
    fn collect_tokens(&self, _tokens: &mut Vec<SemToken>) {}
}

impl SemTokens for Codef {
    fn collect_tokens(&self, _tokens: &mut Vec<SemToken>) {}
}

impl SemTokens for Let {
    fn collect_tokens(&self, _tokens: &mut Vec<SemToken>) {}
}
