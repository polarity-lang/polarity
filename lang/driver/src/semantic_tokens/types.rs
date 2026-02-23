//! The different token types and modifiers supported by the server.

use lsp_types::{SemanticToken, SemanticTokenType, SemanticTokensLegend};
use polarity_lang_miette_util::{ToMiette, codespan::Span};
use url::Url;

use crate::Database;

/// Describes which semantic tokens and modifiers are emitted by the server.
/// This legend is used because the actual semantic tokens only contain an index into
/// this legend to save space.
pub fn token_legend() -> SemanticTokensLegend {
    SemanticTokensLegend { token_types: vec![SemanticTokenType::ENUM], token_modifiers: vec![] }
}

/// Semantic token modifiers are set using a bitmap.
/// If no token modifiers are used we therefore use the empty bitmap.
const SEMANTIC_TOKEN_MODIFIER_NONE: u32 = 0;

/// The different semantic token types that we are using.
/// They have to be converted to index positions in the legend before we transmit them.
pub enum TokenType {
    DataType,
    CodataType,
}

impl TokenType {
    /// The index of this token type in the legend.
    pub fn index(self) -> u32 {
        match self {
            TokenType::DataType => 0,
            TokenType::CodataType => 0,
        }
    }
}

/// The internal representation of semantic tokens that we use in the server.
/// This representation uses absolute positions.
pub struct SemToken {
    pub span: Span,
    pub typ: TokenType,
}

/// Takes a vector of `SemToken` which contain `Span`'s, i.e. absolute positions,
/// and converts them to a vector of `SemanticToken` which use a relative encoding.
/// That is, the position of each `SemanticToken` is relative to the token preceding
/// it in the vector.
pub fn convert_sem_tokens(db: &Database, uri: &Url, toks: Vec<SemToken>) -> Vec<SemanticToken> {
    let mut output: Vec<SemanticToken> = Vec::new();

    let file = db.files.get_unless_stale(uri).unwrap();

    let mut prev_line: u32 = 0;
    let mut prev_character: u32 = 0;

    for tok in toks {
        let pos = file.location(tok.span.start).unwrap();

        let st = SemanticToken {
            delta_line: pos.line - prev_line,
            delta_start: if pos.line != prev_line { 0 } else { pos.character - prev_character },
            length: u32::try_from(tok.span.to_miette().len()).unwrap(),
            token_type: tok.typ.index(),
            token_modifiers_bitset: SEMANTIC_TOKEN_MODIFIER_NONE,
        };
        output.push(st);
        prev_line = pos.line;
        prev_character = pos.character;
    }

    output
}
