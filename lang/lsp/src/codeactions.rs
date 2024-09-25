//! Implementation of code actions provided by the LSP server
use std::collections::HashMap;
use tower_lsp::{jsonrpc, lsp_types::*};

use driver::Xfunc;

use super::conversion::*;
use super::server::*;

pub async fn code_action(
    server: &Server,
    params: CodeActionParams,
) -> jsonrpc::Result<Option<CodeActionResponse>> {
    let text_document = params.text_document;
    let range = params.range;

    server
        .client
        .log_message(MessageType::INFO, format!("Code action request: {}", text_document.uri))
        .await;

    let mut db = server.database.write().await;
    let span_start = db.location_to_index(&text_document.uri, range.start.from_lsp());
    let span_end = db.location_to_index(&text_document.uri, range.end.from_lsp());
    let span = span_start.and_then(|start| span_end.map(|end| codespan::Span::new(start, end)));
    let item = span.and_then(|span| db.item_at_span(&text_document.uri, span));

    if let Some(item) = item {
        let Ok(Xfunc { title, edits }) = db.xfunc(&text_document.uri, item.type_name()) else {
            return Ok(None);
        };
        let edits = edits
            .into_iter()
            .map(|edit| TextEdit {
                range: db.span_to_locations(&text_document.uri, edit.span).unwrap().to_lsp(),
                new_text: edit.text,
            })
            .collect();

        let mut changes = HashMap::new();
        changes.insert(text_document.uri, edits);

        let res = vec![CodeActionOrCommand::CodeAction(CodeAction {
            title,
            kind: Some(CodeActionKind::REFACTOR_REWRITE),
            edit: Some(WorkspaceEdit { changes: Some(changes), ..Default::default() }),
            ..Default::default()
        })];

        Ok(Some(res))
    } else {
        Ok(Some(vec![]))
    }
}
