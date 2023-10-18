use std::{
    cmp::Ordering,
    fmt,
    path::Path,
};

use serde::Deserialize;
use serde_repr::Deserialize_repr;

use crate::error::Error;

mod sharedlib;

pub use sharedlib::SharedLib as DefaultClient;

pub type Payload = serde_json::Value;

pub trait Client: Sized {
    // REVIEW: how to do load() not on a client?
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error>;
    fn meta(&self) -> Result<Metadata, Error>;
    fn get(&self, component: &str, object: &str) -> Result<Payload, Error>;
    fn set(&self, component: &str, object: &str, payload: &Payload) -> Result<(), Error>;
}

// struct Module<T: Client> {
//     path: String,
//     client: Option<T>,
//     metadata: Metadata,
// }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Metadata {
    pub name: String,
    pub description: Option<String>,
    pub manufacturer: Option<String>,
    #[serde(flatten)]
    pub version: Version,
    // version_info: Option<String>,
    pub components: Vec<String>,
    pub user_account: UserAccount,
    pub lifetime: Lifetime,
    // license_uri: Option<String>,
    // project_uri: Option<String>,
}

#[derive(Debug, Eq, Deserialize, PartialEq)]
pub struct Version {
    #[serde(rename = "VersionMajor", default)]
    major: u32,
    #[serde(rename = "VersionMinor", default)]
    minor: u32,
    #[serde(rename = "VersionPatch", default)]
    patch: u32,
    #[serde(rename = "VersionTweak", default)]
    tweak: u32,
}

/// The lifetime of a module determines how long the module will be kept loaded. Short lifetime
/// modules will be loaded/unloaded for each request. Long lifetime modules will be kept loaded
/// until the platform is restarted.
#[derive(Debug, PartialEq, Deserialize_repr)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum Lifetime {
    Short = 1,
    Long = 2,
}

/// The UID of the user account that the module will be run with. Root (0) is default.
/// UIDs can be found in `/etc/passwd`.
///
/// _Note: UIDs can change/be moved._
#[derive(Debug, PartialEq, Deserialize_repr)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum UserAccount {
    Root = 0,
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => match self.patch.cmp(&other.patch) {
                    Ordering::Equal => self.tweak.cmp(&other.tweak),
                    ordering => ordering,
                },
                ordering => ordering,
            },
            ordering => ordering,
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<&str> for Version {
    fn from(version: &str) -> Self {
        let mut split = version.split('.');

        let major = split.next().unwrap_or("0").parse().unwrap_or(0);
        let minor = split.next().unwrap_or("0").parse().unwrap_or(0);
        let patch = split.next().unwrap_or("0").parse().unwrap_or(0);
        let tweak = split.next().unwrap_or("0").parse().unwrap_or(0);

        Self {
            major,
            minor,
            patch,
            tweak,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "v{}.{}.{}.{}",
            self.major, self.minor, self.patch, self.tweak
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let version = Version::from("1.2.3.4");

        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.tweak, 4);
    }

    fn test_version_cmp(a: &str, b: &str, ordering: Ordering) {
        let a = Version::from(a);
        let b = Version::from(b);

        assert_eq!(a.cmp(&b), ordering);
    }

    #[test]
    fn test_version_cmp_major() {
        test_version_cmp("1.0.0.0", "1.0.0.0", Ordering::Equal);
        test_version_cmp("0.1.0.0", "0.1.0.0", Ordering::Equal);
        test_version_cmp("0.0.1.0", "0.0.1.0", Ordering::Equal);
        test_version_cmp("0.0.0.1", "0.0.0.1", Ordering::Equal);

        test_version_cmp("1.0.0.0", "1.0.0.1", Ordering::Less);
        test_version_cmp("1.0.0.0", "1.0.1.0", Ordering::Less);
        test_version_cmp("1.0.0.0", "1.1.0.0", Ordering::Less);

        test_version_cmp("1.0.0.1", "1.0.0.0", Ordering::Greater);
        test_version_cmp("1.0.1.0", "1.0.0.0", Ordering::Greater);
        test_version_cmp("1.1.0.0", "1.0.0.0", Ordering::Greater);
    }
}
