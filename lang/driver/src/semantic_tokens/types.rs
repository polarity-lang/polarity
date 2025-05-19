//! The different token types and modifiers supported by the server.

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