//! Implementation of the goto-definition functionality of the LSP server

use codespan::Span;
use tower_lsp::{jsonrpc, lsp_types::*};

use super::conversion::*;
use super::server::*;
use driver::*;

pub async fn goto_definition(
    server: &Server,
    params: GotoDefinitionParams,
) -> jsonrpc::Result<Option<GotoDefinitionResponse>> {
    let pos_params = params.text_document_position_params;
    let text_document = pos_params.text_document;

    server
        .client
        .log_message(
            MessageType::INFO,
            format!("GotoDefinition request: {}", text_document.uri.from_lsp()),
        )
        .await;

    let pos = pos_params.position;
    let mut db = server.database.write().await;
    let info = db.location_to_index(&text_document.uri.from_lsp(), pos.from_lsp());
    let info = match info {
        Some(idx) => db.hoverinfo_at_index(&text_document.uri.from_lsp(), idx).await,
        None => None,
    };
    let res = info.and_then(|info| info_to_jump(&db, info));
    Ok(res)
}

fn info_to_jump(db: &Database, info: Info) -> Option<GotoDefinitionResponse> {
    let (uri, span) = info.content.to_jump_target()?;
    let jump_location = span_to_location(&span, &uri, db)?;
    Some(GotoDefinitionResponse::Scalar(jump_location))
}

fn span_to_location(span: &Span, uri: &Uri, db: &Database) -> Option<Location> {
    let range = db.span_to_locations(&uri.from_lsp(), *span).map(ToLsp::to_lsp)?;
    Some(Location { uri: uri.clone(), range })
}

trait ToJumpTarget {
    fn to_jump_target(&self) -> Option<(Uri, Span)>;
}

impl ToJumpTarget for InfoContent {
    fn to_jump_target(&self) -> Option<(Uri, Span)> {
        match self {
            InfoContent::TypeCtorInfo(i) => i.to_jump_target(),
            InfoContent::CallInfo(i) => i.to_jump_target(),
            InfoContent::DotCallInfo(i) => i.to_jump_target(),
            InfoContent::UseInfo(i) => i.to_jump_target(),
            _ => None,
        }
    }
}

impl ToJumpTarget for UseInfo {
    fn to_jump_target(&self) -> Option<(Uri, Span)> {
        Some((self.uri.to_lsp(), Span::default()))
    }
}

impl ToJumpTarget for TypeCtorInfo {
    fn to_jump_target(&self) -> Option<(Uri, Span)> {
        let TypeCtorInfo { definition_site, .. } = self;
        definition_site.clone().map(|(url, span)| (url.to_lsp(), span))
    }
}

impl ToJumpTarget for CallInfo {
    fn to_jump_target(&self) -> Option<(Uri, Span)> {
        let CallInfo { definition_site, .. } = self;
        definition_site.clone().map(|(url, span)| (url.to_lsp(), span))
    }
}

impl ToJumpTarget for DotCallInfo {
    fn to_jump_target(&self) -> Option<(Uri, Span)> {
        let DotCallInfo { definition_site, .. } = self;
        definition_site.clone().map(|(url, span)| (url.to_lsp(), span))
    }
}
