use std::sync::Arc;
mod collect;
pub mod types;

use crate::{Database, semantic_tokens::collect::SemTokens};
use ast::Module;
use lsp_types::SemanticToken;
use types::{SemToken, convert_sem_tokens};
use url::Url;

pub fn compute_semantic_tokens(
    db: &Database,
    uri: &Url,
    module: Arc<Module>,
) -> Vec<SemanticToken> {
    // We first collect the tokens with absolute positions.
    let mut tokens: Vec<SemToken> = Vec::new();
    module.collect_tokens(&mut tokens);

    // Then we convert them to relative positions.
    convert_sem_tokens(db, uri, tokens)
}
