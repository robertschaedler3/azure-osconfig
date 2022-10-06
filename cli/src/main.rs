use clap::{Parser, Subcommand};
mod command;

#[derive(Debug, Parser)]
#[clap(name = "osc")]
#[clap(about = "azure-osconfig command line", long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    command: Command,
    // TODO: flag for validating return payloads against thier mim schemas
    // TODO: flag for verbose logging
    // TODO: check the status of the platform
}

#[derive(Debug, Subcommand)]
enum Command {
    // #[clap(arg_required_else_help = true)]
    // Platform(command::Platform),

    #[clap(arg_required_else_help = true)]
    Generate(command::generate::Arguments),
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let _ = match args.command {
        // Command::Platform(command) => {
        //     command::platform::execute(command)
        // }
        Command::Generate(arguments) => {
            command::generate::execute(arguments)
        }
    };

    // TODO: handle results from the commands
}
