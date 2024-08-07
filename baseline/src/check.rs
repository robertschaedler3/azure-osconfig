use std::path::Path;

use anyhow::Result;
use baseline_codegen::check;
use regex::Regex;
use serde::de::DeserializeOwned;

pub trait Check: DeserializeOwned {
    // type Params;

    fn check(&self) -> Result<bool>;
}

// Checks if the given file exists
// #[check]
// pub fn file_exists(path: String) -> Result<bool> {
//     let path = Path::new(&self.path);
//     Ok(path.exists())
// }

// REVIEW: use wildcards in path instead of adding a regex pattern?
#[check]
pub fn file_exists(path: String, pattern: Option<String>) -> Result<bool> {
    let path = Path::new(&self.path);
    if path.is_dir() && self.pattern.is_some() {
        // If the path is a directory, check if any file matches the pattern
        let dir = std::fs::read_dir(path)?;
        let re = Regex::new(&self.pattern.as_ref().unwrap())?;

        for entry in dir {
            let entry = entry?;
            let filename = entry.file_name().into_string().unwrap();
            if re.is_match(&filename) {
                return Ok(true);
            }
        }

        Ok(false)
    } else {
        // Otherwise, check if the file exists
        return Ok(path.exists());
    }

    // TODO: warn if there is a pattern and the path is not a directory
}

#[check]
pub fn file_contains(path: String) -> Result<bool> {
    Ok(false)
}


// instead of using a macro to simply define each check, these should also be aggregated into an enum

