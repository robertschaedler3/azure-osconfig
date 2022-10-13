use clap::Args;

pub mod test;
// pub mod platform;

// #[derive(Debug, Args)]
// #[clap(args_conflicts_with_subcommands = true)]
// struct Platform {
//     #[clap(subcommand)]
//     command: platform::Command,
//     // TODO: options for platform commands:
//     // -a, --all
//     // -q, --quiet
//     // -v, --verbose
// }

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct Test {
    #[clap(subcommand)]
    command: test::Command,
}

