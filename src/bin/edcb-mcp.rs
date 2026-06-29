use std::process;

use rmcp::{ServiceExt, transport::stdio};

use edcb_tools::mcp::{EdcbMcpServer, ServerConfigAction};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = match ServerConfigAction::from_env_args() {
        Ok(ServerConfigAction::Run(config)) => config,
        Ok(ServerConfigAction::Help(text) | ServerConfigAction::Version(text)) => {
            print!("{text}");
            return Ok(());
        }
        Err(error) => {
            eprintln!("error: {error}");
            process::exit(2);
        }
    };
    let service = EdcbMcpServer::new(config).serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
