//! Implementation of the goto-definition functionality of the LSP server

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
    let pos = pos_params.position;
    let db = server.database.read().await;
    let index = db.get(text_document.uri.as_str()).unwrap();
    let info = index.location_to_index(pos.from_lsp()).and_then(|idx| index.info_at_index(idx));
    let res = info.and_then(|info| info_to_jump(&index, text_document.uri, info));
    Ok(res)
}

fn info_to_jump(index: &DatabaseView, uri: Url, info: Info) -> Option<GotoDefinitionResponse> {
    info.content.to_jump_content(index, uri).map(GotoDefinitionResponse::Scalar)
}

trait ToJumpContent {
    fn to_jump_content(&self, index: &DatabaseView, uri: Url) -> Option<Location>;
}

impl ToJumpContent for InfoContent {
    fn to_jump_content(&self, index: &DatabaseView, uri: Url) -> Option<Location> {
        match self {
            InfoContent::VariableInfo(i) => i.to_jump_content(index, uri),
            InfoContent::TypeCtorInfo(i) => i.to_jump_content(index, uri),
            InfoContent::CallInfo(i) => i.to_jump_content(index, uri),
            InfoContent::DotCallInfo(i) => i.to_jump_content(index, uri),
            InfoContent::TypeUnivInfo(i) => i.to_jump_content(index, uri),
            InfoContent::HoleInfo(i) => i.to_jump_content(index, uri),
            InfoContent::AnnoInfo(i) => i.to_jump_content(index, uri),
        }
    }
}

impl ToJumpContent for VariableInfo {
    fn to_jump_content(&self, _index: &DatabaseView, _uri: Url) -> Option<Location> {
        None
    }
}

impl ToJumpContent for TypeCtorInfo {
    fn to_jump_content(&self, index: &DatabaseView, uri: Url) -> Option<Location> {
        let TypeCtorInfo { target_span } = self;
        match target_span {
            Some(span) => {
                let rng = index.span_to_locations(*span).map(ToLsp::to_lsp);
                let location = Location { uri, range: rng.unwrap() };
                Some(location)
            }
            None => None,
        }
    }
}

impl ToJumpContent for CallInfo {
    fn to_jump_content(&self, _index: &DatabaseView, _uri: Url) -> Option<Location> {
        None
    }
}

impl ToJumpContent for DotCallInfo {
    fn to_jump_content(&self, _index: &DatabaseView, _uri: Url) -> Option<Location> {
        None
    }
}

impl ToJumpContent for TypeUnivInfo {
    fn to_jump_content(&self, _index: &DatabaseView, _uri: Url) -> Option<Location> {
        None
    }
}

impl ToJumpContent for HoleInfo {
    fn to_jump_content(&self, _index: &DatabaseView, _uri: Url) -> Option<Location> {
        None
    }
}

impl ToJumpContent for AnnoInfo {
    fn to_jump_content(&self, _index: &DatabaseView, _uri: Url) -> Option<Location> {
        None
    }
}
