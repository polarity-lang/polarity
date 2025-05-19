//! The different token types and modifiers supported by the server.

use lsp_types::{SemanticToken, SemanticTokenType, SemanticTokensLegend};
use polarity_lang_miette_util::codespan::Span;

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


pub enum TokenType {
    DataType,
    CodataType,
}

impl TokenType {
    pub fn index(self) -> u32 {
        match self {
            TokenType::DataType => 0,
            TokenType::CodataType => 0,
        }
    }
}

pub struct SemToken {
    pub span: Span,
    pub typ: TokenType,
}

pub fn convert_sem_tokens(_toks: Vec<SemToken>) -> Vec<SemanticToken> {
    todo!()
}