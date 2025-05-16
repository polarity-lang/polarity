use std::sync::Arc;

use lsp_types::SemanticToken;
use polarity_lang_ast::Module;

pub fn compute_semantic_tokens(_module: Arc<Module>) -> Vec<SemanticToken> {
    todo!()
}
