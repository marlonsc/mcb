#![allow(missing_docs)]
extern crate mcb_providers;

use clap::{Parser, Subcommand};
use mcb::cli::{ServeArgs, ValidateArgs};

#[derive(Parser, Debug)]
#[command(name = "mcb")]
#[command(about = "MCP Context Browser - Semantic Code Search Server")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(alias = "server")]
    Serve(ServeArgs),
    Validate(ValidateArgs),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Serve(args) => args.execute().await,
        Command::Validate(args) => {
            let result = args.execute()?;
            if result.failed() {
                std::process::exit(1);
            }
            Ok(())
        }
    }
}
