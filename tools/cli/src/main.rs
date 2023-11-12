use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use cli::Module;
use colored::*;
use colored_json::to_colored_json_auto;

const DEFAULT_BIN_DIR: &str = "/workspaces/azure-osconfig/build/lib"; // REVIEW: only for demo

#[derive(Debug, Parser)]
#[command(name = "osc")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Verbose output
    #[clap(short, long)]
    verbose: bool,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Gets the reported value of a property
    Get {
        /// The component name
        component: String,

        /// The property name
        property: String,
    },

    /// Sets the value of a property
    Set {
        /// The component name
        component: String,

        /// The property name
        property: String,

        /// The property value JSON
        value: String,
    },

    /// Lists all components
    List,
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    let modules = std::fs::read_dir(DEFAULT_BIN_DIR)
        .context("Unable to read modules directory")?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();

            if path.is_file() && path.extension().map(|ext| ext == "so").unwrap_or(false) {
                Some(path)
            } else {
                None
            }
        })
        .filter_map(|path| {
            let module = Module::new(path);
            if let Err(err) = &module {
                log::warn!("Unable to load module: {}", err);
            }
            module.ok()
        })
        .collect::<Vec<_>>();

    let args = Cli::parse();

    match args.command {
        Commands::Get {
            component,
            property,
        } => {
            let module = modules
                .iter()
                .find(|module| module.compnents().contains(&component))
                .context("Unable to find module")?;
            let value = module
                .get(&component, &property)
                .context("Unable to get property")?;

            if args.verbose {
                println!("{} - ({}.{})", module.path.file_name().unwrap().to_string_lossy().bright_black(), component.blue(), property.blue());
                // println!("{}", to_colored_json_auto(&module.meta).unwrap());
            }

            println!("{}", to_colored_json_auto(&value).unwrap());
        }
        Commands::Set {
            component,
            property,
            value,
        } => {
            let module = modules
                .iter()
                .find(|module| module.compnents().contains(&component))
                .context("Unable to find module")?;
            let value = serde_json::from_str(&value).context("Unable to parse value")?;
            module
                .set(&component, &property, &value)
                .context("Unable to set property")?;
        }
        Commands::List => {
            println!("Listing all components");
        }
    }

    Ok(())
}
