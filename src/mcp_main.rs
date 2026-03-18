use lint::McpServer;
use clap::Parser;

#[derive(Parser)]
#[command(name = "lint-mcp")]
#[command(about = "MCP server for lint tool", long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "127.0.0.1")]
    host: String,

    #[arg(short, long, default_value = "8080")]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let server = McpServer::new(cli.host, cli.port);
    server.run().await
}
