//! Implementation of code actions provided by the LSP server
use std::collections::HashMap;
use tower_lsp::{jsonrpc, lsp_types::*};

use query::Xfunc;

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

    let db = server.database.read().await;
    let index = db.get(text_document.uri.as_str()).unwrap();
    let span_start = index.location_to_index(range.start.from_lsp());
    let span_end = index.location_to_index(range.end.from_lsp());
    let span = span_start.and_then(|start| span_end.map(|end| codespan::Span::new(start, end)));
    let item = span.and_then(|span| index.item_at_span(span));

    if let Some(item) = item {
        let Xfunc { title, edits } = index.xfunc(item.type_name()).unwrap();
        let edits = edits
            .into_iter()
            .map(|edit| TextEdit {
                range: index.span_to_locations(edit.span).unwrap().to_lsp(),
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
