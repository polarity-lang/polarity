//! Implementation of the type-on-hover functionality of the LSP server

use ast::CallKind;
use ast::DotCallKind;
use driver::*;
use tower_lsp::{jsonrpc, lsp_types::*};

use super::conversion::*;
use super::server::*;

// The implementation of the hover functionality that gets called by the LSP server.
pub async fn hover(server: &Server, params: HoverParams) -> jsonrpc::Result<Option<Hover>> {
    let pos_params = params.text_document_position_params;
    let text_document = pos_params.text_document;

    server
        .client
        .log_message(MessageType::INFO, format!("Hover request: {}", text_document.uri.from_lsp()))
        .await;

    let pos = pos_params.position;
    let mut db = server.database.write().await;
    let info = db.location_to_index(&text_document.uri.from_lsp(), pos.from_lsp());

    let info = match info {
        Some(idx) => db.hoverinfo_at_index(&text_document.uri.from_lsp(), idx).await,
        None => None,
    };

    let res = info.map(|info| info_to_hover(&db, &text_document.uri, info));
    Ok(res)
}

fn info_to_hover(db: &Database, uri: &Uri, info: Info) -> Hover {
    let range = db.span_to_locations(&uri.from_lsp(), info.span).map(ToLsp::to_lsp);
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

fn add_doc_comment(builder: &mut Vec<MarkedString>, doc: Option<Vec<String>>) {
    if let Some(doc) = doc {
        builder.push(MarkedString::String("---".to_owned()));
        for d in doc {
            builder.push(MarkedString::String(d))
        }
    }
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
            InfoContent::UseInfo(i) => i.to_hover_content(),
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
        let TypeCtorInfo { name, doc, .. } = self;
        let mut content: Vec<MarkedString> = Vec::new();
        content.push(MarkedString::String(format!("Type constructor: `{}`", name)));
        add_doc_comment(&mut content, doc);
        HoverContents::Array(content)
    }
}

impl ToHoverContent for CallInfo {
    fn to_hover_content(self) -> HoverContents {
        let CallInfo { kind, typ, name, doc, .. } = self;
        let mut content: Vec<MarkedString> = Vec::new();
        content.push(match kind {
            CallKind::Constructor => MarkedString::String(format!("Constructor: `{}`", name)),
            CallKind::Codefinition => MarkedString::String(format!("Codefinition: `{}`", name)),
            CallKind::LetBound => MarkedString::String(format!("Let-bound definition: `{}`", name)),
        });
        add_doc_comment(&mut content, doc);
        content.push(MarkedString::String("---".to_owned()));
        content.push(string_to_language_string(typ));
        HoverContents::Array(content)
    }
}

impl ToHoverContent for DotCallInfo {
    fn to_hover_content(self) -> HoverContents {
        let DotCallInfo { kind, name, typ, doc, .. } = self;
        let mut content: Vec<MarkedString> = Vec::new();
        content.push(match kind {
            DotCallKind::Destructor => MarkedString::String(format!("Destructor: `{}`", name)),
            DotCallKind::Definition => MarkedString::String(format!("Definition: `{}`", name)),
        });
        add_doc_comment(&mut content, doc);
        content.push(MarkedString::String("---".to_owned()));
        content.push(string_to_language_string(typ));
        HoverContents::Array(content)
    }
}

impl ToHoverContent for TypeUnivInfo {
    fn to_hover_content(self) -> HoverContents {
        let content: Vec<MarkedString> = vec![
            MarkedString::String("Universe: `Type`".to_owned()),
            MarkedString::String("---".to_owned()),
            MarkedString::String(
                "The impredicative universe whose terms are types or the universe `Type` itself."
                    .to_owned(),
            ),
        ];
        HoverContents::Array(content)
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
        let HoleInfo { metavar, goal, ctx, args, metavar_state } = self;
        if let Some(ctx) = ctx {
            let mut value = String::new();
            match metavar {
                Some(mv) => value.push_str(&format!("Hole: `{}`\n\n", mv)),
                None => value.push_str("Hole: `?`\n\n"),
            }
            goal_to_markdown(&goal, &mut value);
            value.push_str("\n\n");
            ctx_to_markdown(&ctx, &mut value);
            value.push_str("\n\nArguments:\n\n");
            let args_str = args.iter().cloned().map(comma_separated).map(|s| format!("({})", s));
            let args_str = format!("({})", comma_separated(args_str));
            value.push_str(&args_str);
            if let Some(solution) = metavar_state {
                value.push_str("\n\nSolution:\n\n");
                value.push_str(&solution);
            }
            HoverContents::Markup(MarkupContent { kind: MarkupKind::Markdown, value })
        } else {
            HoverContents::Scalar(string_to_language_string(goal))
        }
    }
}

// Toplevel declarations
//
//

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

// Modules
//
//

impl ToHoverContent for UseInfo {
    fn to_hover_content(self) -> HoverContents {
        let UseInfo { path, .. } = self;
        let content = MarkedString::String(format!("Import module `{}`", path));
        HoverContents::Scalar(content)
    }
}

fn comma_separated<I: IntoIterator<Item = String>>(iter: I) -> String {
    separated(", ", iter)
}

fn separated<I: IntoIterator<Item = String>>(s: &str, iter: I) -> String {
    let vec: Vec<_> = iter.into_iter().collect();
    vec.join(s)
}
