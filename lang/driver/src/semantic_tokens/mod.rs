use std::sync::Arc;

use lsp_types::{SemanticToken, SemanticTokenType, SemanticTokensLegend};
use polarity_lang_ast::{Codata, Codef, Data, Decl, Def, Extern, Let, Module, Note};

/// Describes which semantic tokens and modifiers are emitted by the server.
/// This legend is used because the actual semantic tokens only contain an index into
/// this legend to save space.
pub fn token_legend() -> SemanticTokensLegend {
    SemanticTokensLegend { token_types: vec![SemanticTokenType::TYPE], token_modifiers: vec![] }
}

/// The index of this token type in the legend.
const SEMANTIC_TOKEN_DATATYPE: u32 = 0;

/// Semantic token modifiers are set using a bitmap.
/// If no token modifiers are used we therefore use the empty bitmap.
const SEMANTIC_TOKEN_MODIFIER_NONE: u32 = 0;

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
            Decl::Extern(ext) => ext.semantic_tokens(tokens),
            Decl::Note(note) => note.semantic_tokens(tokens),
            Decl::Data(data) => data.semantic_tokens(tokens),
            Decl::Codata(codata) => codata.semantic_tokens(tokens),
            Decl::Def(def) => def.semantic_tokens(tokens),
            Decl::Codef(codef) => codef.semantic_tokens(tokens),
            Decl::Let(tl_let) => tl_let.semantic_tokens(tokens),
            Decl::Infix(_infix) => {}
        }
    }
}

impl SemTokens for Extern {
    fn semantic_tokens(&self, _tokens: &mut Vec<SemanticToken>) {}
}

impl SemTokens for Note {
    fn semantic_tokens(&self, _tokens: &mut Vec<SemanticToken>) {}
}

impl SemTokens for Data {
    fn semantic_tokens(&self, tokens: &mut Vec<SemanticToken>) {
        let Data { name, .. } = self;
        let st: SemanticToken = SemanticToken {
            delta_line: todo!(),
            delta_start: todo!(),
            length: todo!(),
            token_type: SEMANTIC_TOKEN_DATATYPE,
            token_modifiers_bitset: SEMANTIC_TOKEN_MODIFIER_NONE,
        };
        tokens.push(st);
    }
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
