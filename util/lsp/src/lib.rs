use std::collections::HashMap;

use tower_lsp::{jsonrpc, lsp_types::*, LanguageServer};

use async_lock::RwLock;

use source::{Database, File, Xfunc};

pub fn capabilities() -> lsp::ServerCapabilities {
    let document_symbol_provider = Some(lsp::OneOf::Left(true));

    let text_document_sync = {
        let options = lsp::TextDocumentSyncOptions {
            open_close: Some(true),
            change: Some(lsp::TextDocumentSyncKind::FULL),
            ..Default::default()
        };
        Some(lsp::TextDocumentSyncCapability::Options(options))
    };

    let hover_provider = Some(HoverProviderCapability::Simple(true));

    let code_action_provider = Some(lsp::CodeActionProviderCapability::Simple(true));

    lsp::ServerCapabilities {
        text_document_sync,
        document_symbol_provider,
        hover_provider,
        code_action_provider,
        ..Default::default()
    }
}

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
            index.location_to_index(pos.into_location()).and_then(|idx| index.info_at_index(idx));
        let res = info.map(|info| {
            let range =
                info.span.and_then(|span| index.span_to_locations(span)).map(IntoRange::into_range);
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
        let span_start = index.location_to_index(range.start.into_location());
        let span_end = index.location_to_index(range.end.into_location());
        let span = span_start.and_then(|start| span_end.map(|end| codespan::Span::new(start, end)));
        let item = span.and_then(|span| index.item_at_span(span));

        if let Some(item) = item {
            let Xfunc { title, edits } = index.xfunc(item.name()).unwrap();
            let edits = edits
                .into_iter()
                .map(|edit| TextEdit {
                    range: index.span_to_locations(edit.span).unwrap().into_range(),
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

trait Extract {
    type Target;

    fn extract(self) -> Self::Target;
}

impl<T> Extract for Result<T, T> {
    type Target = T;

    fn extract(self) -> Self::Target {
        match self {
            Ok(x) => x,
            Err(x) => x,
        }
    }
}

trait IntoLocation {
    fn into_location(self) -> codespan::Location;
}

trait IntoPosition {
    fn into_position(self) -> Position;
}

trait IntoRange {
    fn into_range(self) -> Range;
}

trait IntoSpan {
    fn into_span(self) -> codespan::Span;
}

impl IntoLocation for Position {
    fn into_location(self) -> codespan::Location {
        codespan::Location { line: self.line.into(), column: self.character.into() }
    }
}

impl IntoPosition for codespan::Location {
    fn into_position(self) -> Position {
        Position { line: self.line.into(), character: self.column.into() }
    }
}

impl IntoRange for (codespan::Location, codespan::Location) {
    fn into_range(self) -> Range {
        Range { start: self.0.into_position(), end: self.1.into_position() }
    }
}
