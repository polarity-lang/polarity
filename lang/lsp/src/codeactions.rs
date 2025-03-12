//! Implementation of code actions provided by the LSP server

use std::collections::HashMap;

use tower_lsp_server::{jsonrpc, lsp_types::*};

use driver::{Database, Item, Xfunc};

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
        let mut res = vec![];

        if let Some(action) = xfunc_action(&mut db, &text_document, &item).await {
            res.push(action);
        }

        if let Some(action) = lifting_action(&mut db, &text_document, &item).await {
            res.push(action);
        }

        Ok(Some(res))
    } else {
        Ok(Some(vec![]))
    }
}

async fn xfunc_action(
    db: &mut Database,
    text_document: &TextDocumentIdentifier,
    item: &Item,
) -> Option<CodeActionOrCommand> {
    let Ok(Xfunc { title, edits }) =
        db.xfunc(&text_document.uri.from_lsp(), item.type_name()).await
    else {
        return None;
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
    changes.insert(text_document.uri.clone(), edits);

    let res = CodeActionOrCommand::CodeAction(CodeAction {
        title,
        kind: Some(CodeActionKind::REFACTOR_REWRITE),
        edit: Some(WorkspaceEdit { changes: Some(changes), ..Default::default() }),
        ..Default::default()
    });

    Some(res)
}

async fn lifting_action(
    db: &mut Database,
    text_document: &TextDocumentIdentifier,
    item: &Item,
) -> Option<CodeActionOrCommand> {
    let Ok(edits) = db.lift(&text_document.uri.from_lsp(), item.type_name()).await else {
        return None;
    };

    if edits.is_empty() {
        return None;
    }

    let edits = edits
        .into_iter()
        .map(|edit| TextEdit {
            range: db.span_to_locations(&text_document.uri.from_lsp(), edit.span).unwrap(),
            new_text: edit.text,
        })
        .collect();

    #[allow(clippy::mutable_key_type)]
    let mut changes = HashMap::new();
    changes.insert(text_document.uri.clone(), edits);

    let res = CodeActionOrCommand::CodeAction(CodeAction {
        title: format!("Lift {}", item.type_name()),
        kind: Some(CodeActionKind::REFACTOR_REWRITE),
        edit: Some(WorkspaceEdit { changes: Some(changes), ..Default::default() }),
        ..Default::default()
    });

    Some(res)
}
