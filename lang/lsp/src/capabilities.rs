use tower_lsp::lsp_types::*;

pub fn capabilities() -> ServerCapabilities {
    let text_document_sync = {
        let options = TextDocumentSyncOptions {
            open_close: Some(true),
            change: Some(TextDocumentSyncKind::FULL),
            ..Default::default()
        };
        Some(TextDocumentSyncCapability::Options(options))
    };

    let hover_provider = Some(HoverProviderCapability::Simple(true));

    let code_action_provider = Some(CodeActionProviderCapability::Simple(true));

    let document_formatting_provider = Some(OneOf::Left(true));

    let definition_provider = Some(OneOf::Left(true));

    ServerCapabilities {
        text_document_sync,
        hover_provider,
        code_action_provider,
        document_formatting_provider,
        definition_provider,
        ..Default::default()
    }
}
