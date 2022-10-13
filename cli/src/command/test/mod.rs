use std::{path::PathBuf};

use anyhow::anyhow;
use clap::{Args, Subcommand};
use colored::Colorize;

use crate::Result;
use fixture::Fixture;

mod fixture;
mod log;

#[derive(Debug, Args)]
pub struct Arguments {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Module {
        #[clap(value_parser = valid_path)]
        path: PathBuf,

        #[clap(default_value = "/usr/lib/osconfig")]
        #[clap(value_parser = valid_path)]
        bin: PathBuf,

        // TODO: filter/run specific tests
    },
}

pub fn execute(args: Arguments) -> Result<()> {
    match args.command {
        Command::Module { path, bin } => test_module(path, bin),
    }
}

fn test_module(path: PathBuf, bin: PathBuf) -> Result<()> {
    let definition = path.join("test.yml");

    // TODO: use model in payload validation for get/set
    // let interface = path.join("interface.json");
    // let model = interface::Model::from_file(interface)?;

    let (setup, teardown, fixture) = Fixture::from_file(&definition)?;

    println!("");
    println!("test definition: {}", definition.to_string_lossy());

    // TODO: print more details about the test run (tests, modules, etc.)

    if let Some(script) = setup {
        // TODO: need a way to set all the tests to skipped if setup fails
        script.execute()?;
        // println!("setup {}", "complete".bright_green());
    }

    // REVIEW: what is the best way to handle a failure here (teardown should always run) ?
    if let Err(err) = fixture.run(bin) {
        println!("{}", err);
    }

    if let Some(script) = teardown {
        script.execute()?;
        // println!("teardown {}", "complete".bright_green());
    }

    Ok(())
}

fn valid_path(s: &str) -> Result<PathBuf> {
    let path = std::env::current_dir()?.join(s);
    let path = path.canonicalize()?;
    let path_str = path.to_str().ok_or(anyhow!("Invalid path"))?;

    if !path.exists() {
        Err(anyhow!("{} does not exist", path_str))
    } else if !path.is_dir() {
        Err(anyhow!("{} is not a directory", path_str))
    } else {
        Ok(path)
    }
}
