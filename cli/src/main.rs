use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};

mod command;
// mod config;

// TODO: this needs to be more generic
// const DEFAULT_MIM_PATH: &str = "/home/rschaedler/git/azure-osconfig/src/modules/mim";

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
    // Config(Config),

    #[clap(arg_required_else_help = true)]
    Platform(Platform),
    // TODO:
    // Status(Status),
    // Module(Module),
    // Mim(Mim),
}

// #[derive(Debug, Args)]
// #[clap(args_conflicts_with_subcommands = true)]
// struct Config {
//     #[clap(subcommand)]
//     command: ConfigCommand,
// }

// #[derive(Debug, Subcommand)]
// enum ConfigCommand {
//     #[clap(arg_required_else_help = true)]
//     Set {
//         #[clap(value_parser)]
//         key: String,

//         // TODO: value should be an AppConfig struct
//         #[clap(value_parser)]
//         value: String,
//     },

//     #[clap(arg_required_else_help = false)]
//     Get {
//         #[clap(value_parser)]
//         key: String,
//     },
//     // #[clap(arg_required_else_help = false)]
//     // List,
// }

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
struct Platform {
    #[clap(subcommand)]
    command: PlatformCommand,
    // TODO: options for platform commands:
    // -a, --all
    // -q, --quiet
    // -v, --verbose
}

#[derive(Debug, Subcommand)]
enum PlatformCommand {
    #[clap(arg_required_else_help = true)]
    Status,
    Get {
        #[clap(value_parser)]
        component: Option<String>,

        #[clap(value_parser)]
        object: Option<String>,
    },
    Set {
        #[clap(value_parser)]
        component: Option<String>,

        #[clap(value_parser)]
        object: Option<String>,

        #[clap(value_parser)]
        value: Option<String>,
    },
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct Config {
    client_name: Option<String>,
    max_payload_size: Option<usize>,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    match args.command {
        // Command::Config(sub_command) => match sub_command.command {
        //     ConfigCommand::Get { key } => {
        //         let config = load::<AppConfig>(CONFIG_NAME).unwrap();
        //         let value = serde_json::to_value(config).unwrap();
        //         let value = value.get(&key).unwrap();
        //         println!("{}", value);
        //     }
        //     ConfigCommand::Set { key, value } => {
        //         let mut config = load::<AppConfig>(CONFIG_NAME).unwrap();
        //         match key.as_ref() {
        //             "client_name" => {
        //                 config.client_name = Some(value);
        //             }
        //             "max_payload_size" => {
        //                 config.max_payload_size = Some(value.parse::<usize>().unwrap());
        //             }
        //             _ => {
        //                 println!("Unknown key: {}", key);
        //             }
        //         }
        //         store::<AppConfig>(CONFIG_NAME, config).unwrap();
        //     }
        // },
        Command::Platform(sub_command) => {
            // TODO: fix this cfg module location
            // let cfg = config::Config::load();
            let handler = command::platform::Handler::new();
            match sub_command.command {
                PlatformCommand::Get { component, object } => {
                    handler.get(component, object).await;
                }
                PlatformCommand::Set {
                    component,
                    object,
                    value,
                } => {
                    handler.set(component, object, value).await;
                }
                PlatformCommand::Status => {
                    println!("Status");
                }
        }},
    }
}
