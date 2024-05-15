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
    let index = db.get(&text_document.uri).unwrap();
    let info =
        index.location_to_index(pos.from_lsp()).and_then(|idx| index.hoverinfo_at_index(idx));
    let res = info.and_then(|info| info_to_jump(&index, info));
    Ok(res)
}

fn info_to_jump(index: &DatabaseView, info: Info) -> Option<GotoDefinitionResponse> {
    info.content.to_jump_content(index).map(GotoDefinitionResponse::Scalar)
}

trait ToJumpContent {
    fn to_jump_content(&self, index: &DatabaseView) -> Option<Location>;
}

impl ToJumpContent for InfoContent {
    fn to_jump_content(&self, index: &DatabaseView) -> Option<Location> {
        match self {
            InfoContent::VariableInfo(i) => i.to_jump_content(index),
            InfoContent::TypeCtorInfo(i) => i.to_jump_content(index),
            InfoContent::CallInfo(i) => i.to_jump_content(index),
            InfoContent::DotCallInfo(i) => i.to_jump_content(index),
            InfoContent::TypeUnivInfo(i) => i.to_jump_content(index),
            InfoContent::HoleInfo(i) => i.to_jump_content(index),
            InfoContent::AnnoInfo(i) => i.to_jump_content(index),
            InfoContent::LocalComatchInfo(i) => i.to_jump_content(index),
            InfoContent::LocalMatchInfo(i) => i.to_jump_content(index),
            InfoContent::DefInfo(i) => i.to_jump_content(index),
            InfoContent::CodefInfo(i) => i.to_jump_content(index),
            InfoContent::LetInfo(i) => i.to_jump_content(index),
            InfoContent::DataInfo(i) => i.to_jump_content(index),
            InfoContent::CodataInfo(i) => i.to_jump_content(index),
            InfoContent::CtorInfo(i) => i.to_jump_content(index),
            InfoContent::DtorInfo(i) => i.to_jump_content(index),
        }
    }
}

impl ToJumpContent for VariableInfo {
    fn to_jump_content(&self, _index: &DatabaseView) -> Option<Location> {
        None
    }
}

impl ToJumpContent for TypeCtorInfo {
    fn to_jump_content(&self, index: &DatabaseView) -> Option<Location> {
        let TypeCtorInfo { target_span, .. } = self;
        target_span.as_ref().map(|span| {
            let rng = index.span_to_locations(span.1).map(ToLsp::to_lsp);
            Location { uri: span.0.clone(), range: rng.unwrap() }
        })
    }
}

impl ToJumpContent for CallInfo {
    fn to_jump_content(&self, _index: &DatabaseView) -> Option<Location> {
        None
    }
}

impl ToJumpContent for DotCallInfo {
    fn to_jump_content(&self, _index: &DatabaseView) -> Option<Location> {
        None
    }
}

impl ToJumpContent for TypeUnivInfo {
    fn to_jump_content(&self, _index: &DatabaseView) -> Option<Location> {
        None
    }
}

impl ToJumpContent for HoleInfo {
    fn to_jump_content(&self, _index: &DatabaseView) -> Option<Location> {
        None
    }
}

impl ToJumpContent for AnnoInfo {
    fn to_jump_content(&self, _index: &DatabaseView) -> Option<Location> {
        None
    }
}

impl ToJumpContent for LocalMatchInfo {
    fn to_jump_content(&self, _index: &DatabaseView) -> Option<Location> {
        None
    }
}

impl ToJumpContent for LocalComatchInfo {
    fn to_jump_content(&self, _index: &DatabaseView) -> Option<Location> {
        None
    }
}

impl ToJumpContent for DefInfo {
    fn to_jump_content(&self, _index: &DatabaseView) -> Option<Location> {
        None
    }
}

impl ToJumpContent for CodefInfo {
    fn to_jump_content(&self, _index: &DatabaseView) -> Option<Location> {
        None
    }
}

impl ToJumpContent for LetInfo {
    fn to_jump_content(&self, _index: &DatabaseView) -> Option<Location> {
        None
    }
}

impl ToJumpContent for DataInfo {
    fn to_jump_content(&self, _index: &DatabaseView) -> Option<Location> {
        None
    }
}

impl ToJumpContent for CodataInfo {
    fn to_jump_content(&self, _index: &DatabaseView) -> Option<Location> {
        None
    }
}

impl ToJumpContent for CtorInfo {
    fn to_jump_content(&self, _index: &DatabaseView) -> Option<Location> {
        None
    }
}

impl ToJumpContent for DtorInfo {
    fn to_jump_content(&self, _index: &DatabaseView) -> Option<Location> {
        None
    }
}
