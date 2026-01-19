use common_tools::create_server;
use rust_mcp_sdk::{error::SdkResult, McpServer};
use tracing::{error, info};

fn initialize_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}

#[tokio::main]
async fn main() -> SdkResult<()> {
    initialize_tracing();

    info!("Starting Common Tools MCP Server");

    let server = create_server()
        .map_err(|e| {
            error!("Failed to create server: {}", e);
            std::process::exit(1);
        })
        .unwrap();

    if let Err(start_error) = server.start().await {
        error!("Server start error: {}", start_error);
        eprintln!(
            "{}",
            start_error
                .rpc_error_message()
                .unwrap_or(&start_error.to_string())
        );
    }

    Ok(())
}
