use tower_lsp::{LspService, Server};

#[derive(clap::Args)]
pub struct Args {}

pub fn exec(_: Args) -> miette::Result<()> {
    let stdin = async_std::io::stdin();
    let stdout = async_std::io::stdout();
    let (service, messages) = LspService::new(lsp_server::Server::new);
    tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap().block_on(async {
        Server::new(stdin, stdout, messages).serve(service).await;
    });
    Ok(())
}
