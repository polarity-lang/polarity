use async_lock::RwLock;
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::{LanguageServer, jsonrpc, lsp_types::*};

use driver::Database;
#[cfg(not(target_arch = "wasm32"))]
use driver::{FileSource, FileSystemSource, InMemorySource};

use crate::conversion::{FromLsp, ToLsp};

use super::capabilities::*;
use super::diagnostics::*;

pub struct Server {
    pub client: tower_lsp_server::Client,
    pub database: RwLock<Database>,
}

impl Server {
    pub fn new(client: tower_lsp_server::Client) -> Self {
        let database = Database::in_memory();
        Self::with_database(client, database)
    }

    pub fn with_database(client: tower_lsp_server::Client, database: Database) -> Self {
        Server { client, database: RwLock::new(database) }
    }
}

impl LanguageServer for Server {
    async fn initialize(&self, params: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        let capabilities = capabilities();
        #[cfg(not(target_arch = "wasm32"))]
        // FIXME: Use `workspace_folders` instead of `root_uri`.
        // `root_uri` is deprecated in in favor of `workspace_folders`, see:
        // https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#initialize
        #[allow(deprecated)]
        if let Some(root_uri) = params.root_uri {
            let root_path =
                root_uri.from_lsp().to_file_path().map_err(|_| jsonrpc::Error::internal_error())?;
            let source = InMemorySource::new().fallback_to(FileSystemSource::new(root_path));
            let mut database = self.database.write().await;
            let source_mut = database.file_source_mut();
            *source_mut = Box::new(source);
        }
        // prevent unused variable warning when compiled for wasm
        let _ = params;
        Ok(InitializeResult { capabilities, ..InitializeResult::default() })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "server initialized!").await;
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        self.client.log_message(MessageType::INFO, "server shutdown!").await;
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let text_document = params.text_document;
        let mut db = self.database.write().await;

        self.client
            .log_message(
                MessageType::INFO,
                format!("Opened file: {}", text_document.uri.from_lsp()),
            )
            .await;

        let source_mut = db.file_source_mut();
        assert!(source_mut.manage(&text_document.uri.from_lsp()));
        source_mut.write_string(&text_document.uri.from_lsp(), &text_document.text).await.unwrap();

        let res = db.ast(&text_document.uri.from_lsp()).await.map(|_| ());
        let diags = db.diagnostics(&text_document.uri.from_lsp(), res).await;
        self.send_diagnostics(diags).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let text_document = params.text_document;
        let mut db = self.database.write().await;

        self.client
            .log_message(
                MessageType::INFO,
                format!("Closed file: {}", text_document.uri.from_lsp()),
            )
            .await;

        let source_mut = db.file_source_mut();
        source_mut.forget(&text_document.uri.from_lsp());
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let text_document = params.text_document;
        let mut content_changes = params.content_changes;

        self.client
            .log_message(
                MessageType::INFO,
                format!("Changed file: {}", text_document.uri.from_lsp()),
            )
            .await;

        let mut db = self.database.write().await;
        let text = content_changes.drain(0..).next().unwrap().text;

        let source_mut = db.file_source_mut();
        assert!(source_mut.manage(&text_document.uri.from_lsp()));
        source_mut.write_string(&text_document.uri.from_lsp(), &text).await.unwrap();

        db.invalidate(&text_document.uri.from_lsp()).await;

        let res = db.ast(&text_document.uri.from_lsp()).await.map(|_| ());
        let diags = db.diagnostics(&text_document.uri.from_lsp(), res).await;

        self.send_diagnostics(diags).await;
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> jsonrpc::Result<Option<GotoDefinitionResponse>> {
        super::gotodefinition::goto_definition(self, params).await
    }

    async fn hover(&self, params: HoverParams) -> jsonrpc::Result<Option<Hover>> {
        super::hover::hover(self, params).await
    }

    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> jsonrpc::Result<Option<CodeActionResponse>> {
        super::codeactions::code_action(self, params).await
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        super::format::formatting(self, params).await
    }
}

impl Server {
    pub(crate) async fn send_diagnostics(&self, diags: DiagnosticsPerUri) {
        for (uri, diags) in diags {
            self.client.publish_diagnostics(uri.to_lsp(), diags, None).await;
        }
    }
}
