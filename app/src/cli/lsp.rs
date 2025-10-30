use tower_lsp_server::{LspService, Server};

#[derive(clap::Args)]
pub struct Args {}

pub async fn exec(_: Args) -> Result<(), Vec<miette::Report>> {
    let stdin = async_std::io::stdin();
    let stdout = async_std::io::stdout();
    let (service, messages) = LspService::new(polarity_lang_lsp_server::Server::new);
    Server::new(stdin, stdout, messages).serve(service).await;
    Ok(())
}
