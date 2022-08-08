use std::error::Error;

use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use colored_json::to_colored_json_auto;
use dialoguer::{theme::ColorfulTheme, Confirm, Select};

use osc::{
    client::Client,
    mim::{Primitive, Schema, TypeSchema},
};

mod config;
use config::Config;

#[derive(Debug, Parser)]
#[clap(name = "osc")]
#[clap(about = "azure-osconfig command line", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
    // TODO: flag for validating return payloads against thier mim schemas
    // TODO: flag for verbose logging
}

#[derive(Debug, Subcommand)]
enum Commands {
    Session(Session),
    #[clap(arg_required_else_help = false)]
    Get {
        #[clap(value_parser)]
        component: Option<String>,

        #[clap(value_parser)]
        object: Option<String>,

        /// Get all reported objects for the given component (ignores component and object names)
        #[clap(short, long, value_parser, default_value_t = false)]
        all: bool,
    },
    #[clap(arg_required_else_help = false)]
    Set {
        #[clap(value_parser)]
        component: Option<String>,

        #[clap(value_parser)]
        object: Option<String>,

        // TODO: see how this can be parsed better
        // TODO: Prompt for values based on schema
        #[clap(short, long, value_parser)]
        value: Option<String>,
        // SetDesired: flag -a
    },
    // TODO:
    // List a schema for a component, object, setting
    // List all components, objects, settings
    // Settings: mim directory, etc
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
struct Session {
    #[clap(subcommand)]
    command: Option<SessionCommands>,
}

#[derive(Debug, Subcommand)]
enum SessionCommands {
    #[clap(arg_required_else_help = true)]
    Create {
        #[clap(value_parser)]
        name: String,

        #[clap(short, long, value_parser, default_value_t = 0)]
        max_payload_size: u64,
    },
    #[clap(arg_required_else_help = true)]
    Delete {
        #[clap(value_parser)]
        name: String,
    },
    // TODO: list saved sessions
    // #[clap(arg_required_else_help = true)]
    // List,
}

#[derive(Debug)]
struct Handler {
    client: Client,
    schema: Schema,
    config: Config,
}

impl Handler {
    pub fn new(cfg: Config) -> Self {
        Self {
            client: Client::default(),
            // TODO: use path from config to load schema, only when necessary (ie when executing a command that requires it)
            schema: Schema::load("/home/rschaedler/git/azure-osconfig/src/modules/mim".to_string())
                .unwrap(),
            config: cfg,
        }
    }

    fn select<T>(&self, message: &str, mut options: Vec<T>, report: bool) -> T
    where
        T: std::fmt::Display + Clone + Ord,
    {
        options.sort();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(message)
            .default(0)
            .items(&options[..])
            .report(report)
            .interact()
            .unwrap();
        options[selection].clone()
    }

    fn prompt(
        &self,
        name: &String,
        schema: TypeSchema,
    ) -> serde_json::Value {
        match schema {
            TypeSchema::Primitive(primitive) => match primitive {
                Primitive::String => {
                    let value = dialoguer::Input::new()
                        .with_prompt(name)
                        .report(false)
                        .interact()
                        .unwrap();
                    serde_json::Value::String(value)
                }
                Primitive::Integer => {
                    let value: i32 = dialoguer::Input::new()
                        .with_prompt(name)
                        .report(false)
                        .interact()
                        .unwrap();
                    serde_json::Value::Number(serde_json::Number::from(value))
                }
                Primitive::Boolean => {
                    let value = Select::with_theme(&ColorfulTheme::default())
                        .with_prompt(name)
                        .default(0)
                        .items(&["true", "false"])
                        .report(false)
                        .interact()
                        .unwrap();
                    serde_json::Value::Bool(value == 0)
                }
            },
            TypeSchema::Enum(enum_) => {
                let options = enum_
                    .values()
                    .iter()
                    .map(|v| v.name())
                    .collect::<Vec<String>>();
                let value = self.select(name, options.clone(), false);
                serde_json::Value::Number(serde_json::Number::from(
                    options.iter().position(|v| v == &value).unwrap() as i64,
                ))
            }
            TypeSchema::Object(object) => {
                let mut value = serde_json::Value::Object(serde_json::Map::new());
                for sub_schema in object.fields().iter() {
                    value[sub_schema.name()] =
                        self.prompt(&sub_schema.name(), sub_schema.schema().clone());
                }
                value
            }
            TypeSchema::Array(array) => {
                let sub_schema = array.schema();
                let mut value = serde_json::Value::Array(vec![]);
                loop {
                    let item = self.prompt(name, sub_schema.clone());
                    value.as_array_mut().unwrap().push(item);
                    let continue_ = Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt("Add another item?")
                        .report(false)
                        .interact()
                        .unwrap();

                    if !continue_ {
                        break;
                    }
                }
                value
            }
        }
    }

    fn get_session(&self) -> String {
        // TODO: if a session does not exist, prompt for one
        // let session = self.prompt("Select a session", sessions);
        self.config.session.clone()
    }

    fn print_result(&self, result: Result<String, Box<dyn Error + Send + Sync>>) {
        match result {
            Ok(result) => {
                // TODO: if json cannot be parsed, print error and do not colorize
                let result: serde_json::Value = serde_json::from_str(&result).unwrap();
                // println!("{}", serde_json::to_string_pretty(&result).unwrap());
                println!("{}", to_colored_json_auto(&result).unwrap());
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    pub async fn get(&self, component: Option<String>, object: Option<String>) {
        let component = component
            .unwrap_or_else(|| self.select("component", self.schema.reported_components(), false));
        let object = object.unwrap_or_else(|| {
            self.select("object", self.schema.reported_objects(&component), false)
        });
        let session = self.get_session();
        let result = self.client.get(session, component, object).await;
        // TODO: if the result is an enum, print its value from the mim schema
        self.print_result(result);
    }

    pub async fn get_reported(&self) {
        let session = self.get_session();
        let result = self.client.get_reported(session).await;
        self.print_result(result);
    }

    pub async fn set(
        &self,
        component: Option<String>,
        object: Option<String>,
        value: Option<String>,
    ) {
        let component = component
            .unwrap_or_else(|| self.select("component", self.schema.desired_components(), true));
        let object = object.unwrap_or_else(|| {
            self.select("object", self.schema.desired_objects(&component), true)
        });
        let session = self.get_session();

        println!(
            "Provide a value for {}.{}:",
            component.bright_blue(),
            object.bright_blue()
        );

        let value = {
            if let Some(value) = value {
                // TODO: validate the payload against the mim schema
                serde_json::from_str(&value).unwrap()
            } else {
                let schema = self.schema.setting(&component, &object);
                self.prompt(&object, schema)
            }
        };

        println!("{}", serde_json::to_string_pretty(&value).unwrap());

        let _ = self.client.set(session, component, object, value).await;
        // MpiSet does not return a result string
        // self.print_result(result);
    }
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let client = Client::default();
    let cfg: Config = confy::load("osc_cli").unwrap();
    let handler = Handler::new(cfg);

    match args.command {
        Commands::Session(session) => {
            match session.command {
                Some(SessionCommands::Create {
                    name,
                    max_payload_size,
                }) => {
                    let session_id = client.open(name.clone(), max_payload_size).await;
                    match session_id {
                        Ok(session) => {
                            // REVIEW: Should max_payload_size be logged too?
                            let _ = confy::store("osc_cli", Config{session: session.clone()});
                            println!("Created session: {} ({})", name.bold().green(), session.italic());
                        }
                        Err(e) => {
                            eprintln!("Failed to open session for {}", name.bold().red());
                            eprintln!("{}", e);
                        }
                    }
                }
                Some(SessionCommands::Delete { name }) => {
                    // TODO: need to pass client name to make sure the session id is the correct one
                    let session_id = "";
                    let _ = client.close(session_id.to_string()).await;
                    println!(
                        "Deleted session: {} ({})",
                        name.bold().red(),
                        session_id.italic()
                    );
                }
                _ => unreachable!(),
            }
        }
        Commands::Get {
            component,
            object,
            all,
        } => {
            // TODO: what if component is not None (should get all reported objects for a given component)
            if all {
                handler.get_reported().await;
            } else {
                handler.get(component, object).await;
            }
        }
        Commands::Set {
            component,
            object,
            value,
        } => {
            handler.set(component, object, value).await;
        }
    }
}
