use std::sync::Arc;

use ast::Module;
use lsp_types::SemanticToken;

pub fn compute_semantic_tokens(_module: Arc<Module>) -> Vec<SemanticToken> {
    todo!()
}
