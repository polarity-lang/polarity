//! Implementation of the goto-definition functionality of the LSP server

use codespan::Span;
use tower_lsp::{jsonrpc, lsp_types::*};

use super::conversion::*;
use super::server::*;
use query::*;

pub async fn goto_definition(
    server: &Server,
    params: GotoDefinitionParams,
) -> jsonrpc::Result<Option<GotoDefinitionResponse>> {
    let pos_params = params.text_document_position_params;
    let text_document = pos_params.text_document;

    server
        .client
        .log_message(MessageType::INFO, format!("GotoDefinition request: {}", text_document.uri))
        .await;

    let pos = pos_params.position;
    let mut db = server.database.write().await;
    let Ok(index) = db.open_uri(&text_document.uri) else {
        return Ok(None);
    };
    let info =
        index.location_to_index(pos.from_lsp()).and_then(|idx| index.hoverinfo_at_index(idx));
    let res = info.and_then(|info| info_to_jump(&index, info));
    Ok(res)
}

fn info_to_jump(index: &DatabaseViewMut, info: Info) -> Option<GotoDefinitionResponse> {
    let (uri, span) = info.content.to_jump_target()?;
    let jump_location = span_to_location(&span, uri, index)?;
    Some(GotoDefinitionResponse::Scalar(jump_location))
}

fn span_to_location(span: &Span, uri: Url, index: &DatabaseViewMut) -> Option<Location> {
    let range = index.span_to_locations(*span).map(ToLsp::to_lsp)?;
    Some(Location { uri, range })
}
trait ToJumpTarget {
    fn to_jump_target(&self) -> Option<(Url, Span)>;
}

impl ToJumpTarget for InfoContent {
    fn to_jump_target(&self) -> Option<(Url, Span)> {
        match self {
            InfoContent::TypeCtorInfo(i) => i.to_jump_target(),
            InfoContent::CallInfo(i) => i.to_jump_target(),
            InfoContent::DotCallInfo(i) => i.to_jump_target(),
            _ => None,
        }
    }
}

impl ToJumpTarget for TypeCtorInfo {
    fn to_jump_target(&self) -> Option<(Url, Span)> {
        let TypeCtorInfo { definition_site, .. } = self;
        definition_site.clone()
    }
}

impl ToJumpTarget for CallInfo {
    fn to_jump_target(&self) -> Option<(Url, Span)> {
        let CallInfo { definition_site, .. } = self;
        definition_site.clone()
    }
}

impl ToJumpTarget for DotCallInfo {
    fn to_jump_target(&self) -> Option<(Url, Span)> {
        let DotCallInfo { definition_site, .. } = self;
        definition_site.clone()
    }
}
