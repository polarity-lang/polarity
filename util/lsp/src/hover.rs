//! Implementation of the type-on-hover functionality of the LSP server

use tower_lsp::{jsonrpc, lsp_types::*};

use query::{Binder, Ctx, DatabaseView, HoverInfo, HoverInfoContent};

use super::conversion::*;
use super::server::*;

// The implementation of the hover functionality that gets called by the LSP server.
pub async fn hover(server: &Server, params: HoverParams) -> jsonrpc::Result<Option<Hover>> {
    let pos_params = params.text_document_position_params;
    let text_document = pos_params.text_document;
    let pos = pos_params.position;
    let db = server.database.read().await;
    let index = db.get(text_document.uri.as_str()).unwrap();
    let info =
        index.location_to_index(pos.from_lsp()).and_then(|idx| index.hoverinfo_at_index(idx));
    let res = info.map(|info| info_to_hover(&index, info));
    Ok(res)
}

fn info_to_hover(index: &DatabaseView, info: HoverInfo) -> Hover {
    let range = index.span_to_locations(info.span).map(ToLsp::to_lsp);
    let contents = info.content.to_hover_content();
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

trait ToHoverContent {
    fn to_hover_content(self) -> HoverContents;
}

impl ToHoverContent for HoverInfoContent {
    fn to_hover_content(self) -> HoverContents {
        match self.ctx {
            Some(ctx) => {
                let mut value = String::new();
                goal_to_markdown(&self.typ, &mut value);
                value.push_str("\n\n");
                ctx_to_markdown(&ctx, &mut value);
                HoverContents::Markup(MarkupContent { kind: MarkupKind::Markdown, value })
            }
            None => HoverContents::Scalar(MarkedString::LanguageString(LanguageString {
                language: "pol".to_owned(),
                value: self.typ,
            })),
        }
    }
}
