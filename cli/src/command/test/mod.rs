use std::{path::PathBuf};

use anyhow::anyhow;
use clap::{Args, Subcommand};
// use colored::Colorize;

use crate::Result;

// TODO: fix this namespacing
// use fixture::Fixture;
use definition::Definition;

mod definition;
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
        // TODO: default to current working directory
        #[clap(value_parser = valid_path)]
        path: PathBuf,

        #[clap(default_value = "/usr/lib/osconfig")]
        #[clap(value_parser = valid_path)]
        bin: PathBuf,

        // TODO:
        // - filter/run specific tests
        // - run tests using the platform vs directly loading modules
    },
}

pub fn execute(args: Arguments) -> Result<()> {
    match args.command {
        Command::Module { path, bin } => test_module(path, bin),
    }
}

fn test_module(path: PathBuf, bin: PathBuf) -> Result<()> {
    let yml = path.join("test.yml");

    if !yml.exists() {
        return Err(anyhow!("Test definition file not found: {}", yml.display()));
    }

    // TODO: use model in payload validation for get/set
    // let interface = path.join("interface.json");
    // let model = interface::Model::from_file(interface)?;

    let definition = Definition::from_file(&yml)?;
    let (setup, teardown, fixture) = definition.into_parts();

    // TODO: print more details about the test run (tests, modules, etc.)

    if let Some(script) = setup {
        // TODO: need a way to set all the tests to skipped if setup fails
        script.execute()?;
    }

    // REVIEW: if a test fails catastrophically, teardown should still run after
    fixture.run(bin)?;

    if let Some(script) = teardown {
        // REVIEW: should anything special happen if the teardown fails?
        script.execute()?;
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
