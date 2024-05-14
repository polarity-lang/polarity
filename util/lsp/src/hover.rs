//! Implementation of the type-on-hover functionality of the LSP server

use query::*;
use syntax::ast::CallKind;
use syntax::ast::DotCallKind;
use tower_lsp::{jsonrpc, lsp_types::*};

use super::conversion::*;
use super::server::*;

// The implementation of the hover functionality that gets called by the LSP server.
pub async fn hover(server: &Server, params: HoverParams) -> jsonrpc::Result<Option<Hover>> {
    let pos_params = params.text_document_position_params;
    let text_document = pos_params.text_document;

    server
        .client
        .log_message(MessageType::INFO, format!("Hover request: {}", text_document.uri))
        .await;

    let pos = pos_params.position;
    let db = server.database.read().await;
    let index = db.get(&text_document.uri).unwrap();
    let info =
        index.location_to_index(pos.from_lsp()).and_then(|idx| index.hoverinfo_at_index(idx));
    let res = info.map(|info| info_to_hover(&index, info));
    Ok(res)
}

fn info_to_hover(index: &DatabaseView, info: Info) -> Hover {
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

impl ToHoverContent for InfoContent {
    fn to_hover_content(self) -> HoverContents {
        match self {
            InfoContent::VariableInfo(i) => i.to_hover_content(),
            InfoContent::TypeCtorInfo(i) => i.to_hover_content(),
            InfoContent::CallInfo(i) => i.to_hover_content(),
            InfoContent::DotCallInfo(i) => i.to_hover_content(),
            InfoContent::TypeUnivInfo(i) => i.to_hover_content(),
            InfoContent::LocalMatchInfo(i) => i.to_hover_content(),
            InfoContent::LocalComatchInfo(i) => i.to_hover_content(),
            InfoContent::HoleInfo(i) => i.to_hover_content(),
            InfoContent::AnnoInfo(i) => i.to_hover_content(),
            InfoContent::DataInfo(i) => i.to_hover_content(),
            InfoContent::CtorInfo(i) => i.to_hover_content(),
            InfoContent::CodataInfo(i) => i.to_hover_content(),
            InfoContent::DtorInfo(i) => i.to_hover_content(),
            InfoContent::DefInfo(i) => i.to_hover_content(),
            InfoContent::CodefInfo(i) => i.to_hover_content(),
            InfoContent::LetInfo(i) => i.to_hover_content(),
        }
    }
}

// Expressions
//
//

impl ToHoverContent for VariableInfo {
    fn to_hover_content(self) -> HoverContents {
        let VariableInfo { typ, name } = self;
        let header = MarkedString::String(format!("Bound variable: `{}`", name));
        let typ = string_to_language_string(typ);
        HoverContents::Array(vec![header, typ])
    }
}

impl ToHoverContent for TypeCtorInfo {
    fn to_hover_content(self) -> HoverContents {
        let TypeCtorInfo { name } = self;
        let header = MarkedString::String(format!("Type constructor: `{}`", name));
        HoverContents::Array(vec![header])
    }
}

impl ToHoverContent for CallInfo {
    fn to_hover_content(self) -> HoverContents {
        let CallInfo { kind, typ, name } = self;
        let header = match kind {
            CallKind::Constructor => MarkedString::String(format!("Constructor: `{}`", name)),
            CallKind::Codefinition => MarkedString::String(format!("Codefinition: `{}`", name)),
            CallKind::LetBound => MarkedString::String(format!("Let-bound definition: `{}`", name)),
        };

        let typ = string_to_language_string(typ);
        HoverContents::Array(vec![header, typ])
    }
}

impl ToHoverContent for DotCallInfo {
    fn to_hover_content(self) -> HoverContents {
        let DotCallInfo { kind, name, typ } = self;
        let header = match kind {
            DotCallKind::Destructor => MarkedString::String(format!("Destructor: `{}`", name)),
            DotCallKind::Definition => MarkedString::String(format!("Definition: `{}`", name)),
        };
        let typ = string_to_language_string(typ);
        HoverContents::Array(vec![header, typ])
    }
}

impl ToHoverContent for TypeUnivInfo {
    fn to_hover_content(self) -> HoverContents {
        let header = MarkedString::String("Type universe".to_owned());
        HoverContents::Array(vec![header])
    }
}

impl ToHoverContent for AnnoInfo {
    fn to_hover_content(self) -> HoverContents {
        let AnnoInfo { typ } = self;
        let header = MarkedString::String("Annotated term".to_owned());
        let typ = string_to_language_string(typ);
        HoverContents::Array(vec![header, typ])
    }
}

impl ToHoverContent for LocalMatchInfo {
    fn to_hover_content(self) -> HoverContents {
        let LocalMatchInfo { typ } = self;
        let header = MarkedString::String("Local match".to_owned());
        let typ = string_to_language_string(typ);
        HoverContents::Array(vec![header, typ])
    }
}

impl ToHoverContent for LocalComatchInfo {
    fn to_hover_content(self) -> HoverContents {
        let LocalComatchInfo { typ } = self;
        let header = MarkedString::String("Local comatch".to_owned());
        let typ = string_to_language_string(typ);
        HoverContents::Array(vec![header, typ])
    }
}

impl ToHoverContent for HoleInfo {
    fn to_hover_content(self) -> HoverContents {
        let HoleInfo { goal, ctx } = self;
        if let Some(ctx) = ctx {
            let mut value = String::new();
            goal_to_markdown(&goal, &mut value);
            value.push_str("\n\n");
            ctx_to_markdown(&ctx, &mut value);
            HoverContents::Markup(MarkupContent { kind: MarkupKind::Markdown, value })
        } else {
            HoverContents::Scalar(string_to_language_string(goal))
        }
    }
}

// Toplevel declarations
//
//

fn add_doc_comment(builder: &mut Vec<MarkedString>, doc: Option<Vec<String>>) {
    if let Some(doc) = doc {
        builder.push(MarkedString::String("---".to_owned()));
        for d in doc {
            builder.push(MarkedString::String(d))
        }
    }
}

impl ToHoverContent for DataInfo {
    fn to_hover_content(self) -> HoverContents {
        let DataInfo { name, doc, params } = self;
        let mut content: Vec<MarkedString> = Vec::new();
        content.push(MarkedString::String(format!("Data declaration: `{name}`")));
        add_doc_comment(&mut content, doc);
        if !params.is_empty() {
            content.push(MarkedString::String("---".to_owned()).to_owned());
            content.push(MarkedString::String(format!("Parameters: `{}`", params)))
        }
        HoverContents::Array(content)
    }
}

impl ToHoverContent for CtorInfo {
    fn to_hover_content(self) -> HoverContents {
        let CtorInfo { name, doc } = self;
        let mut content: Vec<MarkedString> = Vec::new();
        content.push(MarkedString::String(format!("Constructor: `{}`", name)));
        add_doc_comment(&mut content, doc);
        HoverContents::Array(content)
    }
}

impl ToHoverContent for CodataInfo {
    fn to_hover_content(self) -> HoverContents {
        let CodataInfo { name, doc, params } = self;
        let mut content: Vec<MarkedString> = Vec::new();
        content.push(MarkedString::String(format!("Codata declaration: `{}`", name)));
        add_doc_comment(&mut content, doc);
        if !params.is_empty() {
            content.push(MarkedString::String("---".to_owned()).to_owned());
            content.push(MarkedString::String(format!("Parameters: `{}`", params)))
        }
        HoverContents::Array(content)
    }
}

impl ToHoverContent for DtorInfo {
    fn to_hover_content(self) -> HoverContents {
        let DtorInfo { name, doc } = self;
        let mut content: Vec<MarkedString> = Vec::new();
        content.push(MarkedString::String(format!("Destructor: `{}`", name)));
        add_doc_comment(&mut content, doc);
        HoverContents::Array(content)
    }
}

impl ToHoverContent for DefInfo {
    fn to_hover_content(self) -> HoverContents {
        let header = MarkedString::String("Definition".to_owned());
        HoverContents::Scalar(header)
    }
}

impl ToHoverContent for CodefInfo {
    fn to_hover_content(self) -> HoverContents {
        let header = MarkedString::String("Codefinition".to_owned());
        HoverContents::Scalar(header)
    }
}

impl ToHoverContent for LetInfo {
    fn to_hover_content(self) -> HoverContents {
        let header = MarkedString::String("Let-binding".to_owned());
        HoverContents::Scalar(header)
    }
}
