use clap::{Parser, Subcommand};

mod command;

// TODO: there has to be a better place to put these
pub type Error = anyhow::Error;
pub type Result<T> = anyhow::Result<T>;

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
    Test(command::test::Arguments),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    match args.command {
        // Command::Platform(command) => command::platform::execute(command),
        Command::Test(arguments) => command::test::execute(arguments),
    }
}
