//! Implementation of the goto-definition functionality of the LSP server

use tower_lsp::{jsonrpc, lsp_types::*};

use super::conversion::*;
use super::server::*;

pub async fn goto_definition(
    server: &Server,
    params: GotoDefinitionParams,
) -> jsonrpc::Result<Option<GotoDefinitionResponse>> {
    let pos_params = params.text_document_position_params;
    let text_document = pos_params.text_document;

    server
        .client
        .log_message(
            MessageType::INFO,
            format!("GotoDefinition request: {}", text_document.uri.from_lsp()),
        )
        .await;

    let pos = pos_params.position;
    let mut db = server.database.write().await;
    let info = db.location_to_index(&text_document.uri.from_lsp(), pos);
    let info = match info {
        Some(idx) => db.goto_at_index(&text_document.uri.from_lsp(), idx).await,
        None => None,
    };
    let res = info.and_then(|info| Some(GotoDefinitionResponse::Scalar(info)));
    Ok(res)
}
