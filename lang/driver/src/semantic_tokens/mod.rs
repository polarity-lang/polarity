use std::sync::Arc;
mod collect;
pub mod types;

use crate::semantic_tokens::collect::SemTokens;
use ast::Module;
use lsp_types::SemanticToken;
use types::{SemToken, convert_sem_tokens};

pub fn compute_semantic_tokens(module: Arc<Module>) -> Vec<SemanticToken> {
    // We first collect the tokens with absolute positions.
    let mut tokens: Vec<SemToken> = Vec::new();
    module.collect_tokens(&mut tokens);

    // Then we convert them to relative positions.
    convert_sem_tokens(tokens)
}
