//! Implementation of the type-on-hover functionality of the LSP server

use tower_lsp::{jsonrpc, lsp_types::*};

use query::*;

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

fn string_to_language_string(s: String) -> MarkedString {
    MarkedString::LanguageString(LanguageString { language: "pol".to_owned(), value: s })
}

// Transforming HoverContent to the correct LSP library type.
//
//

trait ToHoverContent {
    fn to_hover_content(self) -> HoverContents;
}

impl ToHoverContent for HoverInfoContent {
    fn to_hover_content(self) -> HoverContents {
        match self {
            HoverInfoContent::GenericInfo(i) => i.to_hover_content(),
            HoverInfoContent::VariableInfo(i) => i.to_hover_content(),
            HoverInfoContent::TypeCtorInfo(i) => i.to_hover_content(),
        }
    }
}

impl ToHoverContent for GenericInfo {
    fn to_hover_content(self) -> HoverContents {
        match self {
            GenericInfo { typ, ctx: Some(ctx) } => {
                let mut value = String::new();
                goal_to_markdown(&typ, &mut value);
                value.push_str("\n\n");
                ctx_to_markdown(&ctx, &mut value);
                HoverContents::Markup(MarkupContent { kind: MarkupKind::Markdown, value })
            }
            GenericInfo { typ, ctx: None } => HoverContents::Scalar(string_to_language_string(typ)),
        }
    }
}

impl ToHoverContent for VariableInfo {
    fn to_hover_content(self) -> HoverContents {
        let VariableInfo { typ } = self;
        let header = MarkedString::String("Bound variable".to_owned());
        let typ = string_to_language_string(typ);
        HoverContents::Array(vec![header, typ])
    }
}

impl ToHoverContent for TypeCtorInfo {
    fn to_hover_content(self) -> HoverContents {
        let TypeCtorInfo { typ } = self;
        let header = MarkedString::String("Type constructor".to_owned());
        let typ = string_to_language_string(typ);
        HoverContents::Array(vec![header, typ])
    }
}
