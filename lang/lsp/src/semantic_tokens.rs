use crate::conversion::FromLsp;

use super::server::*;
use tower_lsp_server::{jsonrpc, lsp_types::*};

pub async fn semantic_tokens_full(
    server: &Server,
    params: SemanticTokensParams,
) -> jsonrpc::Result<Option<SemanticTokensResult>> {
    let text_document = params.text_document;
    server
        .client
        .log_message(
            MessageType::INFO,
            format!("SemanticToken request: {}", text_document.uri.from_lsp()),
        )
        .await;

    let mut _db = server.database.write().await;

    let tokens: SemanticTokens = SemanticTokens { result_id: None, data: vec![] };
    let res: SemanticTokensResult = SemanticTokensResult::Tokens(tokens);
    Ok(Some(res))
}
