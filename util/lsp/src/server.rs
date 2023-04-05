use async_lock::RwLock;
use std::collections::HashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::{jsonrpc, lsp_types::*, LanguageServer};

use printer::{PrintCfg, PrintToString};
use query::{Binder, Ctx, Database, DatabaseView, File, Info, Xfunc};

use super::capabilities::*;
use super::conversion::*;
use super::diagnostics::*;

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
        let mut view = db.add(file);

        let res = view.load();
        let diags = view.query().diagnostics(res);
        self.send_diagnostics(text_document.uri, diags).await;
    }

    async fn did_change(&self, params: lsp::DidChangeTextDocumentParams) {
        let text_document = params.text_document;
        let mut content_changes = params.content_changes;
        let mut db = self.database.write().await;
        let text = content_changes.drain(0..).next().unwrap().text;

        let mut view = db.get_mut(text_document.uri.as_str()).unwrap();

        let res = view.update(text);
        let diags = view.query().diagnostics(res);
        self.send_diagnostics(text_document.uri, diags).await;
    }

    async fn hover(&self, params: HoverParams) -> jsonrpc::Result<Option<Hover>> {
        let pos_params = params.text_document_position_params;
        let text_document = pos_params.text_document;
        let pos = pos_params.position;
        let db = self.database.read().await;
        let index = db.get(text_document.uri.as_str()).unwrap();
        let info = index.location_to_index(pos.from_lsp()).and_then(|idx| index.info_at_index(idx));
        let res = info.map(|info| info_to_hover(&index, info));
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
    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let text_document = params.text_document;
        let db = self.database.read().await;
        let index = db.get(text_document.uri.as_str()).unwrap();
        let prg = match index.ust() {
            Ok(prg) => prg,
            Err(_) => return Ok(None),
        };

        let rng: Range = Range {
            start: Position { line: 0, character: 0 },
            end: Position { line: u32::MAX, character: u32::MAX },
        };

        let cfg = PrintCfg {
            width: 100,
            latex: false,
            omit_decl_sep: false,
            de_bruijn: false,
            indent: 4,
            print_lambda_sugar: true,
        };

        let formatted_prog: String = prg.print_to_string(Some(&cfg));

        let text_edit: TextEdit = TextEdit { range: rng, new_text: formatted_prog };

        Ok(Some(vec![text_edit]))
    }
}

impl Server {
    async fn send_diagnostics(&self, url: Url, diags: Vec<Diagnostic>) {
        self.client.publish_diagnostics(url, diags, None).await;
    }
}

fn info_to_hover(index: &DatabaseView, info: Info) -> Hover {
    let range = info.span.and_then(|span| index.span_to_locations(span)).map(ToLsp::to_lsp);

    let contents = match info.ctx {
        Some(ctx) => {
            let mut value = String::new();
            goal_to_markdown(&info.typ, &mut value);
            value.push_str("\n\n");
            ctx_to_markdown(&ctx, &mut value);
            HoverContents::Markup(MarkupContent { kind: MarkupKind::Markdown, value })
        }
        None => HoverContents::Scalar(MarkedString::LanguageString(LanguageString {
            language: "xfn".to_owned(),
            value: info.typ,
        })),
    };

    Hover { contents, range }
}

fn ctx_to_markdown(ctx: &Ctx, value: &mut String) {
    value.push_str("**Context**\n\n");
    value.push_str("| | |\n");
    value.push_str("|-|-|\n");
    for Binder { name, typ } in ctx.bound.iter().rev().flatten() {
        if name == "_" {
            continue;
        }
        value.push_str("| ");
        value.push_str(name);
        value.push_str(" | `");
        value.push_str(typ);
        value.push_str("` |\n");
    }
}

fn goal_to_markdown(goal_type: &str, value: &mut String) {
    value.push_str("**Goal**\n\n");
    value.push_str("```\n");
    value.push_str(goal_type);
    value.push_str("\n```\n");
}
