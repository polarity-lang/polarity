use super::server::*;
use tower_lsp_server::{jsonrpc, lsp_types::*};

pub async fn semantic_tokens_full(
    _server: &Server,
    _params: SemanticTokensParams,
) -> jsonrpc::Result<Option<SemanticTokensResult>> {
    todo!()
}
