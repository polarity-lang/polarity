//! Implementation of the goto-definition functionality of the LSP server

use tower_lsp::{jsonrpc, lsp_types::*};

use super::server::*;

pub async fn goto_definition(
    _server: &Server,
    params: GotoDefinitionParams,
) -> jsonrpc::Result<Option<GotoDefinitionResponse>> {
    let _ = params;
    Ok(None)
}
