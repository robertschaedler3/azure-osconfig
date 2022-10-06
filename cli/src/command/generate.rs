use std::error::Error;

use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct Arguments {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Module{
        #[clap(short, long, default_value = "rust")]
        language: String,

        interface
    },
}

pub fn execute(args: Arguments) -> Result<(), Box<dyn Error + Send + Sync>> {
    let _ = match args.command {
        Command::Module{ language } => {

        }
    };

    Ok(())
}
