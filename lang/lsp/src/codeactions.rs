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
        .log_message(
            MessageType::INFO,
            format!("Code action request: {}", text_document.uri.as_str()),
        )
        .await;

    let mut db = server.database.write().await;
    let span_start = db.location_to_index(&text_document.uri.from_lsp(), range.start);
    let span_end = db.location_to_index(&text_document.uri.from_lsp(), range.end);
    let span =
        span_start.and_then(|start| span_end.map(|end| miette_util::codespan::Span { start, end }));
    let item = if let Some(span) = span {
        db.item_at_span(&text_document.uri.from_lsp(), span).await
    } else {
        None
    };

    if let Some(item) = item {
        let Ok(Xfunc { title, edits }) =
            db.xfunc(&text_document.uri.from_lsp(), item.type_name()).await
        else {
            return Ok(None);
        };
        let edits = edits
            .into_iter()
            .map(|edit| TextEdit {
                range: db.span_to_locations(&text_document.uri.from_lsp(), edit.span).unwrap(),
                new_text: edit.text,
            })
            .collect();

        #[allow(clippy::mutable_key_type)]
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
