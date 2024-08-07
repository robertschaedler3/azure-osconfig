use std::{fs::File, io::BufReader, path::Path};

use anyhow::Result;
use serde::Deserialize;

use check::Check as _;

mod check;

pub use baseline_codegen::*;

// TODO: this should contain a set of predefinted Audits + their checks
#[derive(Deserialize)]
pub struct Baseline {
    name: String,
    description: Option<String>,
    manufacturer: Option<String>,
    content: Vec<Rule>,
}

#[derive(Deserialize)]
struct Rule {
    name: String,
    description: Option<String>,
    manufacturer: Option<String>,
    audit: Vec<Check>, // remediate: Option<Vec<Remediate>>
}

// REVIEW: macro for aggregating all the different checks into an Autits enum
#[derive(Deserialize)]
#[serde(tag = "check")]
enum Check {
    FileExists(check::FileExists),
}

impl Check {
    fn check(&self) -> Result<bool> {
        match self {
            Self::FileExists(file_exists) => file_exists.check(),
        }
    }
}

pub fn load<P: AsRef<Path>>(path: P) -> Result<Baseline> {
    let path = path.as_ref();
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_yaml::from_reader(reader)?)
}

impl Baseline {
    // TODO: custom result type containing the rules and the status of each check (or error messages if the checks failed to return a status)
    pub fn check_status(&self) -> Status {
        for rule in &self.content {
            for check in &rule.audit {
                if !check.check().unwrap() {
                    return Status::NonCompliant;
                }
            }
        }

        Status::Compliant
    }
}

#[derive(Debug)]
pub enum Status {
    Compliant,
    NonCompliant
}