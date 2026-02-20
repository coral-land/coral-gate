mod command;
mod core;
mod error;
mod shared;

use clap::Parser;
use error::*;

/// TODO: add logging

#[tokio::main]
async fn main() -> Result<()> {
    let cli_arguments = command::structure::Cli::parse();
    // TODO: add tokio select for gracefull shutdown
    match cli_arguments.command {
        command::structure::Commands::Generate(gen_arguments) => {
            command::generate::handle(gen_arguments).await?
        }
        command::structure::Commands::Setup(setup_arguments) => {
            command::setup::handle(setup_arguments).await?
        }
    }

    Ok(())
}
