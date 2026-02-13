use anyhow::Result;
use colored::Colorize;

pub fn run_api(host: String, port: u16, open_browser: bool) -> Result<()> {
    println!("{}", "Starting Folio server with web dashboard...".cyan());

    // Run async server
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(crate::server::api::serve(&host, port, open_browser))?;

    Ok(())
}

pub fn run_mcp(host: String, port: u16) -> Result<()> {
    println!("{}", "Starting Folio MCP server...".cyan());
    println!(
        "{}",
        "This server implements the Model Context Protocol for AI integration.".dimmed()
    );
    println!();

    // Run async server
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(crate::server::mcp::serve(&host, port))?;

    Ok(())
}
