use clap::Subcommand;
use colored::Colorize;
use colored_json::to_colored_json_auto;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Select};
// use osc::log::Log;
use std::error::Error;
// use std::io::{BufRead, BufReader};
// use std::path::Path;
// use std::process::{Command, Stdio};
// use std::sync::mpsc;

use crate::command::Platform;
use osc::platform::Client;
use osc::mim::{Primitive, Schema, TypeSchema};

// TODO: return a Result
pub async fn execute(platform: Platform) {
    // TODO: fix this cfg module location
    // let cfg = config::Config::load();
    let handler = Handler::new();
    match platform.command {
        Command::Get { component, object } => {
            handler.get(component, object).await;
        }
        Command::Set {
            component,
            object,
            value,
        } => {
            handler.set(component, object, value).await;
        }
        Command::Status => {
            // TODO: check the running status of the platform (daemon or app)
        }
        Command::Monitor { errors, verbose } => {
            // let (tx, rx) = channel::<Log>();

            // if verbose {
            //     // Modify "full logging" in the platform config (/etc/osconfig/osconfig.json) to "1"
            //     let mut file = File::open("/etc/osconfig/osconfig.json").unwrap();
            //     let mut contents = String::new();
            //     file.read_to_string(&mut contents).unwrap();
            //     let mut config: Config = serde_json::from_str(&contents).unwrap();
            //     config.full_logging = true;
            //     let config = serde_json::to_string_pretty(&config).unwrap();

            //     let mut file = File::create("/etc/osconfig/osconfig.json").unwrap();
            //     file.write_all(config.as_bytes()).unwrap();
            // }

            // TODO: check if this file has been binplaced before trying to execute it
            // std::thread::spawn(move || {
            //     exec_stream("/usr/bin/osconfig-platform", tx);
            // });

            // loop {
            //     let log = rx.recv().unwrap();
            //     if errors && log.level == log::Level::Error {
            //         println!("{}", log);
            //     } else if !errors {
            //         println!("{}", log);
            //     }
            // }
        }
    }
}

#[derive(Debug, Subcommand)]
enum Command {
    #[clap(arg_required_else_help = true)]
    Status,
    // Open {
    //     #[clap(value_parser)]
    //     client_name: String,
    // },
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
    Monitor {
        // optional flags for only showing errors or warnings
        #[clap(short = 'e', long = "errors")]
        errors: bool,

        #[clap(short = 'v', long = "verbose")]
        verbose: bool,
    },
}

#[derive(Debug)]
pub struct Handler {
    client: Client,
    schema: Schema,
    // config: Config,
}

impl Handler {
    // pub fn new(cfg: Config) -> Self {
    pub fn new() -> Self {
        Self {
            client: Client::default(),
            // TODO: use path from config to load schema, only when necessary (ie when executing a command that requires it)
            schema: Schema::load("/home/rschaedler/git/azure-osconfig/src/modules/mim".to_string())
                .unwrap(),
            // config: cfg,
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

    fn prompt(&self, name: &String, schema: TypeSchema) -> serde_json::Value {
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
            TypeSchema::IntegerEnum(enum_) => {
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
            TypeSchema::StringEnum(enum_) => {
                let options = enum_
                    .values()
                    .iter()
                    .map(|v| v.name())
                    .collect::<Vec<String>>();
                let value = self.select(name, options.clone(), false);
                serde_json::Value::String(value)
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

    // TODO: return Result<> to propagate errors
    async fn get_session(&self) -> String {
        // Check if a session has been stored in the config file
        // if a session does not exist, prompt for one

        // let session = self.prompt("Select a session", sessions);
        // self.config.session.clone()

        self.client.open("blah".to_string(), 4096).await.unwrap()
        // "A32AFE9B-335D-A603-4EA9-819D2DEC0060".to_string()
    }

    // TODO: return Result<> to propagate errors
    fn print_result(&self, result: Result<String, Box<dyn Error + Send + Sync>>) {
        match result {
            Ok(result) => {
                // TODO: if json cannot be parsed, propogate error and do not colorize
                let result: serde_json::Value = serde_json::from_str(&result).unwrap();
                println!("{}", to_colored_json_auto(&result).unwrap());
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    // TODO: return Result<> to propagate errors
    pub async fn get(&self, component: Option<String>, object: Option<String>) {
        let component = component
            .unwrap_or_else(|| self.select("component", self.schema.reported_components(), false));
        let object = object.unwrap_or_else(|| {
            self.select("object", self.schema.reported_objects(&component), false)
        });
        let session = self.get_session().await;
        println!("{}", session);
        let result = self.client.get(session, component, object).await;
        // TODO: if the result is an enum, print its value from the mim schema
        self.print_result(result);
    }

    // TODO: return Result<> to propagate errors
    // pub async fn get_reported(&self) {
    //     let session = self.get_session().await;
    //     let result = self.client.get_reported(session).await;
    //     self.print_result(result);
    // }

    // TODO: return Result<> to propagate errors
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
        let session = self.get_session().await;

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

        let result = self.client.set(session, component, object, value).await;
        // MpiSet does not return a result string
        self.print_result(result);
    }
}

// pub fn exec_stream<P: AsRef<Path>>(binary: P, tx: mpsc::Sender<Log>) {
//     let mut cmd = Command::new(binary.as_ref())
//         .stdout(Stdio::piped())
//         .spawn()
//         .unwrap();

//     {
//         let stdout = cmd.stdout.as_mut().unwrap();
//         let stdout_reader = BufReader::new(stdout);
//         let stdout_lines = stdout_reader.lines();

//         for line in stdout_lines {
//             let trace = Log::trace(line.unwrap()).unwrap();
//             tx.send(trace).unwrap();

//             // TODO: parse the line following the osconfig logging format
//             // TODO: colorize output of logs
//         }
//     }

//     cmd.wait().unwrap();
// }
