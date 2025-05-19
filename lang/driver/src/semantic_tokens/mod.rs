use std::sync::Arc;
pub mod collect;
pub mod types;

use crate::semantic_tokens::collect::SemTokens;
use lsp_types::{SemanticToken, SemanticTokenType, SemanticTokensLegend};
use polarity_lang_ast::{Codata, Codef, Data, Decl, Def, Extern, Let, Module, Note};
use types::{SemToken, convert_sem_tokens};

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
    // We first collect the tokens with absolute positions.
    let mut tokens: Vec<SemToken> = Vec::new();
    module.collect_tokens(&mut tokens);

    // Then we convert them to relative positions.
    convert_sem_tokens(tokens)
}
