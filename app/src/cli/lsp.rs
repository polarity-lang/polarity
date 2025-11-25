use std::path::PathBuf;

use polarity_lang_driver::{Database, InMemorySource};
use tower_lsp_server::{LspService, Server};

use crate::cli::locate_libs::locate_libs;

#[derive(clap::Args)]
pub struct Args {
    #[clap(long)]
    lib_path: Option<Vec<PathBuf>>,
}

pub async fn exec(cmd: Args) -> Result<(), Vec<miette::Report>> {
    let stdin = async_std::io::stdin();
    let stdout = async_std::io::stdout();
    let lib_paths = locate_libs(cmd.lib_path);
    let db = Database::new(InMemorySource::new(), &lib_paths);
    let (service, messages) = LspService::new(|client| polarity_lang_lsp_server::Server::new(client, db));
    Server::new(stdin, stdout, messages).serve(service).await;
    Ok(())
}
