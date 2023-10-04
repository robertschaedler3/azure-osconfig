use std::path::Path;
use std::os::linux::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;

use anyhow::{Context, Result};
use format_serde_error::SerdeError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Baseline {
    #[serde(rename = "BaselineId")]
    id: String,

    // name: String,
    // description: String,

    #[serde(rename = "BaseOrigId")]
    origin: String,

    audits: Vec<Audit>,

    // TODO: allow for other baselines to be imported/included
}

// TODO: struct ComplianceStatus ...

impl Baseline {
    pub fn load(path: &str) -> Result<Baseline> {
        let contents = std::fs::read_to_string(path)?;
        let baseline: Baseline = serde_json::from_str(&contents).map_err(|err| SerdeError::new(contents, err))?;

        log::trace!("Loaded baseline: {:?}", baseline.id);

        Ok(baseline)
    }

    pub fn audit(&self) -> bool {
        log::trace!("Checking baseline: {:?}", self.id);
        self.audits.iter().all(|audit| audit.run())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Audit {
    rule_id: String,
    description: String,
    impact: String,
    remediation: String,
    check: Vec<Check> // TODO: Vec<Check> || Check

    // REVIEW: what is this/can it be better represented?
    // msid: String | Vec<String>,
}

impl Audit {
    pub fn run(&self) -> bool {
        log::trace!("Running audit ({:?}): {:?}", self.rule_id, self.description);
        self.check.iter().all(|check| {
            // TODO: check if the current distro matches the one specified in the check

            log::trace!("Running check: {:?}", check.command);

            // REVIEW: if this can be done without a match statement, it would be WAY more dynamic and VERY clean
            // otherwise this should be generated with a macro
            let result = match &check.command {
                Command::FileExists(check) => check.run(),
                Command::FileExistsRegex(check) => check.run(),
                Command::FileNotExists(check) => check.run(),
                Command::FileStats(check) => check.run(),
                // Command::FileStatsIfExists(check) => check.run(),
                // Command::MatchingConfigValue(check) => check.run(),
                // Command::MatchingLines(check) => check.run(),
                // Command::MatchingLinesAll(check) => check.run(),
                // Command::MatchingLinesAllIfExists(check) => check.run(),
                // Command::MatchingLinesIfExists(check) => check.run(),
                // Command::MatchingLinesInDir(check) => check.run(),
                // Command::MatchingLinesInFiles(check) => check.run(),
                // Command::MatchingLinesSection(check) => check.run(),
                // Command::NoMatchingConfigValue(check) => check.run(),
                // Command::NoMatchingLines(check) => check.run(),
                // Command::NoMatchingLinesIfExists(check) => check.run(),
                _ => {
                    log::error!("Unsupported check: {:?}", check.command);
                    Err(anyhow::anyhow!("Unsupported check: {:?}", check.command))
                }
            };

            // TODO: aggregate errors and return them
            match result {
                Ok(result) => result,
                Err(err) => {
                    log::error!("Error running check: {:?}", err);
                    false
                }
            }
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct Check {
    /// The distro(s) that this check applies to
    distro: String, // REVIEW: only require distro if this audit is an array of checks

    #[serde(flatten)]
    command: Command,

    // TODO: dependency: Vec<Dependency> || Dependency
}

// REVIEW: find a better name for this "Rule" doesnt really make sense
trait Rule {
    fn run(&self) -> Result<bool>;
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "command")]
enum Command {
    // REVIEW: is there a more efficient way to implment groups of similar checks (and organize the code) ?

    FileExists(FileExists),
    FileExistsRegex(FileExistsRegex),
    FileNotExists(FileNotExists),
    FileStats(FileStats),
    MatchingConfigValue(MatchingConfigValue),
    MatchingLines(MatchingLines),
    MatchingLinesAll(MatchingLinesAll),
    MatchingLinesAllIfExists(MatchingLinesAllIfExists),
    MatchingLinesIfExists(MatchingLinesIfExists),
    MatchingLinesInDir(MatchingLinesInDir),
    MatchingLinesInFiles(MatchingLinesInFiles),
    MatchingLinesSection(MatchingLinesSection),
    NoMatchingConfigValue(NoMatchingConfigValue),
    NoMatchingLines(NoMatchingLines),
    NoMatchingLinesIfExists(NoMatchingLinesIfExists),

    // TODO:
}

#[derive(Debug, Serialize, Deserialize)]
struct FileExists {
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileExistsRegex {
    path: String,
    regex: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileNotExists {
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileStats {
    path: String,
    owner: Option<String>,
    group: Option<String>,
    mode: String,

    #[serde(rename = "allow-stricter")]
    allow_stricter: Option<bool>,
}

// #[derive(Debug, Serialize, Deserialize)]
// struct FileStatsIfExists {
//     path: String,
//     owner: Option<String>,
//     group: Option<String>,
//     mode: String, // OR mode_mask

//     #[serde(rename = "allow-stricter")]
//     allow_stricter: Option<bool>,

//     #[serde(rename = "file-type")]
//     file_type: Option<String>,
// }


#[derive(Debug, Serialize, Deserialize)]
struct MatchingConfigValue {
    regex: String,
    exec_command: String
}

#[derive(Debug, Serialize, Deserialize)]
struct MatchingLines {
    path: String,
    regex: String,

    /// Filter the files in the path by this regex
    filter: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MatchingLinesAll {

}

#[derive(Debug, Serialize, Deserialize)]
struct MatchingLinesAllIfExists {

}

#[derive(Debug, Serialize, Deserialize)]
struct MatchingLinesIfExists {

}

#[derive(Debug, Serialize, Deserialize)]
struct MatchingLinesInDir {

}

#[derive(Debug, Serialize, Deserialize)]
struct MatchingLinesInFiles {

}

#[derive(Debug, Serialize, Deserialize)]
struct MatchingLinesSection {

}

#[derive(Debug, Serialize, Deserialize)]
struct NoMatchingConfigValue {

}

#[derive(Debug, Serialize, Deserialize)]
struct NoMatchingLines {

}

#[derive(Debug, Serialize, Deserialize)]
struct NoMatchingLinesIfExists {

}

impl Rule for FileExists {
    fn run(&self) -> Result<bool> {
        let path = Path::new(&self.path);
        Ok(path.exists())
    }
}

impl Rule for FileExistsRegex {
    fn run(&self) -> Result<bool> {
        let path = Path::new(&self.path);

        if !path.exists() || !path.is_dir() {
            return Ok(false);
        }

        // TODO:

        Ok(true)
    }
}

impl Rule for FileNotExists {
    fn run(&self) -> Result<bool> {
        let path = Path::new(&self.path);
        Ok(!path.exists())
    }
}

impl Rule for FileStats {
    fn run(&self) -> Result<bool> {
        let path = Path::new(&self.path);

        if !path.exists() {
            return Ok(false);
        }

        let metadata = path.metadata().context("Failed to get file metadata")?;

        if let Some(owner) = &self.owner {
            let file_owner = metadata
                .st_uid()
                .to_string()
                .parse::<u32>()
                .context("Failed to parse file owner")?;

            let owner = owner.parse::<u32>().context("Failed to parse owner")?;

            if file_owner != owner {
                return Ok(false);
            }
        }

        if let Some(group) = &self.group {
            let file_group = metadata
                .st_gid()
                .to_string()
                .parse::<u32>()
                .context("Failed to parse file group")?;

            let group = group.parse::<u32>().context("Failed to parse group")?;

            if file_group != group {
                return Ok(false);
            }
        }

        let file_mode = metadata.permissions().mode();
        let mode = u32::from_str_radix(&self.mode, 8).context("Failed to parse mode")?;

        if file_mode != mode {
            if let Some(allow_stricter) = self.allow_stricter {
                if allow_stricter && file_mode < mode {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }

        Ok(true)
    }
}


