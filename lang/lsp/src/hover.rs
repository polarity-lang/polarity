//! Implementation of the type-on-hover functionality of the LSP server
use driver::*;
use miette_util::codespan::Span;
use tower_lsp::{jsonrpc, lsp_types::*};

use super::conversion::*;
use super::server::*;

// The implementation of the hover functionality that gets called by the LSP server.
pub async fn hover(server: &Server, params: HoverParams) -> jsonrpc::Result<Option<Hover>> {
    let pos_params = params.text_document_position_params;
    let text_document = pos_params.text_document;

    server
        .client
        .log_message(MessageType::INFO, format!("Hover request: {}", text_document.uri.from_lsp()))
        .await;

    let pos = pos_params.position;
    let mut db = server.database.write().await;
    let info = db.location_to_index(&text_document.uri.from_lsp(), pos);

    let info = match info {
        Some(idx) => db.hoverinfo_at_index(&text_document.uri.from_lsp(), idx).await,
        None => None,
    };

    let res = info.map(|info| info_to_hover(&db, &text_document.uri, info));
    Ok(res)
}

fn info_to_hover(db: &Database, uri: &Uri, contents: (Span, HoverContents)) -> Hover {
    let range = db.span_to_locations(&uri.from_lsp(), contents.0);
    Hover { contents: contents.1, range }
}
