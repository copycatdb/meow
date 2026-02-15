//! # meow
//!
//! A terminal UI (TUI) client for Microsoft SQL Server, powered by
//! [tabby](https://github.com/copycatdb/tabby). Part of the CopyCat ecosystem.
#![allow(unused)]

mod app;
mod cli;
mod commands;
mod db;
mod tui;

use clap::Parser;
use std::path::PathBuf;

/// 🐱 meow — TUI SQL Server client
#[derive(Parser, Debug, Clone)]
#[command(
    name = "meow",
    version,
    about = "🐱 meow — TUI SQL Server client powered by tabby"
)]
pub struct Args {
    /// Server address (host,port)
    #[arg(short = 'S', long = "server", default_value = "localhost,1433")]
    pub server: String,

    /// SQL login username
    #[arg(short = 'U', long = "user")]
    pub user: Option<String>,

    /// SQL login password
    #[arg(short = 'P', long = "password")]
    pub password: Option<String>,

    /// Initial database
    #[arg(short = 'd', long = "database", default_value = "master")]
    pub database: String,

    /// Trust server certificate
    #[arg(long = "trust-cert")]
    pub trust_cert: bool,

    /// Non-interactive CLI mode
    #[arg(long = "cli")]
    pub cli_mode: bool,

    /// Execute SQL from file
    #[arg(short = 'i', long = "input")]
    pub input: Option<PathBuf>,

    /// Write results to file
    #[arg(short = 'o', long = "output")]
    pub output: Option<PathBuf>,

    /// Output format: table, csv, json
    #[arg(long = "format", default_value = "table")]
    pub format: String,
}

impl Args {
    /// Parse the server string into (host, port).
    pub fn parse_server(&self) -> (String, u16) {
        if let Some((host, port_str)) = self.server.split_once(',') {
            let port = port_str.parse::<u16>().unwrap_or(1433);
            (host.to_string(), port)
        } else if let Some((host, port_str)) = self.server.split_once(':') {
            let port = port_str.parse::<u16>().unwrap_or(1433);
            (host.to_string(), port)
        } else {
            (self.server.clone(), 1433)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Determine if we should run in CLI mode:
    // --cli flag, piped stdin, or -i flag
    let is_piped = atty_check();
    if args.cli_mode || is_piped || args.input.is_some() {
        cli::run(args).await?;
    } else {
        tui::run(args).await?;
    }

    Ok(())
}

/// Check if stdin is NOT a terminal (i.e. input is piped).
fn atty_check() -> bool {
    use std::io::IsTerminal;
    !std::io::stdin().is_terminal()
}
