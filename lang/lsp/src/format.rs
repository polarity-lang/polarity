//! Implementation of the formatting functionality of the LSP server
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::lsp_types::*;

use polarity_lang_printer::Print;

use crate::conversion::FromLsp;

use super::server::*;

pub async fn formatting(
    server: &Server,
    params: DocumentFormattingParams,
) -> Result<Option<Vec<TextEdit>>> {
    let text_document = params.text_document;

    server
        .client
        .log_message(
            MessageType::INFO,
            format!("Formatting request: {}", text_document.uri.from_lsp()),
        )
        .await;

    let mut db = server.database.write().await;

    let prg = match db.ust(&text_document.uri.from_lsp()).await {
        Ok(prg) => prg,
        Err(_) => return Ok(None),
    };

    let rng: Range = Range {
        start: Position { line: 0, character: 0 },
        end: Position { line: u32::MAX, character: u32::MAX },
    };

    let formatted_prog: String = prg.print_to_string(None);

    let text_edit: TextEdit = TextEdit { range: rng, new_text: formatted_prog };

    Ok(Some(vec![text_edit]))
}
