use tower_lsp::{LspService, Server};

#[derive(clap::Args)]
pub struct Args {}

pub async fn exec(_: Args) -> miette::Result<()> {
    let stdin = async_std::io::stdin();
    let stdout = async_std::io::stdout();
    let (service, messages) = LspService::new(lsp_server::Server::new);
    Server::new(stdin, stdout, messages).serve(service).await;
    Ok(())
}
