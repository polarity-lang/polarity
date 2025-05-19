use polarity_lang_driver::semantic_tokens::token_legend;
use tower_lsp_server::lsp_types::*;

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

    let semantic_tokens_provider =
        Some(SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
            work_done_progress_options: WorkDoneProgressOptions { work_done_progress: Some(false) },
            legend: token_legend(),
            range: Some(false),
            full: Some(SemanticTokensFullOptions::Bool(true)),
        }));

    ServerCapabilities {
        text_document_sync,
        hover_provider,
        code_action_provider,
        document_formatting_provider,
        definition_provider,
        semantic_tokens_provider,
        ..Default::default()
    }
}
