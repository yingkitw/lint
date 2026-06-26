use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "lint-lsp")]
#[command(about = "Language Server Protocol for lint")]
struct Cli {
    /// LSP communication uses stdio by default
    #[arg(long)]
    stdio: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _cli = Cli::parse();
    lint::LspServer::run().await
}
