use std::collections::HashMap;

use tower_lsp::{jsonrpc, lsp_types::*, LanguageServer};

use async_lock::RwLock;

use data::result::Extract;
use source::{Database, File, Xfunc};

use super::capabilities::*;
use super::conversion::*;

pub struct Server {
    pub client: tower_lsp::Client,
    pub database: RwLock<Database>,
}

impl Server {
    pub fn new(client: tower_lsp::Client) -> Self {
        Server { client, database: RwLock::new(Database::default()) }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Server {
    async fn initialize(&self, _: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        let capabilities = capabilities();
        Ok(InitializeResult { capabilities, ..InitializeResult::default() })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "server initialized!").await;
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: lsp::DidOpenTextDocumentParams) {
        let text_document = params.text_document;
        let mut db = self.database.write().await;
        let file =
            File { name: text_document.uri.to_string(), source: text_document.text, index: true };
        let (msg_t, msg) = db
            .add(file)
            .load()
            .map(|_| format!("Loaded successfully: {}", text_document.uri.as_str()))
            .map(|msg| (MessageType::INFO, msg))
            .map_err(|msg| (MessageType::ERROR, msg.to_string()))
            .extract();
        self.client.log_message(msg_t, msg).await;
    }

    async fn did_change(&self, params: lsp::DidChangeTextDocumentParams) {
        let text_document = params.text_document;
        let mut content_changes = params.content_changes;
        let mut db = self.database.write().await;
        let text = content_changes.drain(0..).next().unwrap().text;
        let (msg_t, msg) = db
            .get_mut(text_document.uri.as_str())
            .unwrap()
            .update(text)
            .map(|_| format!("Loaded successfully: {}", text_document.uri.as_str()))
            .map(|msg| (MessageType::INFO, msg))
            .map_err(|msg| (MessageType::ERROR, msg.to_string()))
            .extract();
        self.client.log_message(msg_t, msg).await;
    }

    async fn hover(&self, params: HoverParams) -> jsonrpc::Result<Option<Hover>> {
        let pos_params = params.text_document_position_params;
        let text_document = pos_params.text_document;
        let pos = pos_params.position;
        let db = self.database.read().await;
        let index = db.get(text_document.uri.as_str()).unwrap();
        let info =
            index.location_to_index(pos.to_codespan()).and_then(|idx| index.info_at_index(idx));
        let res = info.map(|info| {
            let range = info.span.and_then(|span| index.span_to_locations(span)).map(ToLsp::to_lsp);
            Hover {
                contents: HoverContents::Scalar(MarkedString::LanguageString(LanguageString {
                    language: "xfn".to_owned(),
                    value: info.typ,
                })),
                range,
            }
        });
        Ok(res)
    }

    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> jsonrpc::Result<Option<CodeActionResponse>> {
        let text_document = params.text_document;
        let range = params.range;

        let db = self.database.read().await;
        let index = db.get(text_document.uri.as_str()).unwrap();
        let span_start = index.location_to_index(range.start.to_codespan());
        let span_end = index.location_to_index(range.end.to_codespan());
        let span = span_start.and_then(|start| span_end.map(|end| codespan::Span::new(start, end)));
        let item = span.and_then(|span| index.item_at_span(span));

        if let Some(item) = item {
            let Xfunc { title, edits } = index.xfunc(item.name()).unwrap();
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
}
