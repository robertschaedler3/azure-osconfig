use std::{cmp::Ordering, fmt, error::Error};

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use thiserror::Error as ThisError;

// REVIEW: "Prop" vs "Property"

// REVIEW: thiserror ???
pub struct PropertyError {
    message: String,
}

impl PropertyError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl fmt::Display for PropertyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub trait IntoPropertyError {
    /// Performs the custom conversion into a [`FieldError`].
    #[must_use]
    fn into_prop_error(self) -> PropertyError;
}

/// The result of resolving the value of a field of type `T`
pub type PropertyResult<T> = Result<T, PropertyError>;

pub type ExecutionResult = Result<Value, PropertyError>;

pub struct Value(serde_json::Value);

impl Value {
    pub fn new<T: Serialize>(value: T) -> Self {
        Self(serde_json::to_value(value).unwrap())
    }

    pub fn to_string(&self) -> Option<String> {
        serde_json::to_string(&self.0).ok()
    }
}

pub trait IntoPropertyResult<T> {
    #[doc(hidden)]
    fn into_result(self) -> Result<T, PropertyError>;
}

impl<T> IntoPropertyResult<T> for T
where
    T: Serialize,
{
    fn into_result(self) -> Result<T, PropertyError> {
        Ok(self)
    }
}

// TODO: add support for anyhow::Result<T>
impl<T, E> IntoPropertyResult<T> for Result<T, E>
where
    T: IntoPropertyResult<T>,
    E: Error, // REVIEW: this trait works so far... it may need to be narrowed
{
    fn into_result(self) -> PropertyResult<T> {
        self.map_err(|err| PropertyError::new(err.to_string()))
    }
}

pub trait IntoResolvable {
    fn into_resolvable(self) -> PropertyResult<Value>;
}

impl<T> IntoResolvable for PropertyResult<T>
where
    T: Serialize,
{
    fn into_resolvable(self) -> PropertyResult<Value> {
        match self {
            Ok(value) => Ok(Value(serde_json::to_value(value).unwrap())),
            Err(err) => Err(err),
        }
    }
}

#[derive(Debug, ThisError)]
pub enum ModuleError {
    #[error("Unable to find component: {0}")]
    InvalidComponent(String),

    #[error("Unable to find object: {0}")]
    InvalidProperty(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),

    #[error(transparent)]
    Errno(#[from] errno::Errno),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Null(#[from] std::ffi::NulError),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Meta {
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

#[derive(Default, Debug, Eq, Serialize, Deserialize, PartialEq)]
pub struct Version {
    #[serde(rename = "VersionMajor", default)]
    pub major: u32,
    #[serde(rename = "VersionMinor", default)]
    pub minor: u32,
    #[serde(rename = "VersionPatch", default)]
    pub patch: u32,
    #[serde(rename = "VersionTweak", default)]
    pub tweak: u32,
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

/// The lifetime of a module determines how long the module will be kept loaded. Short lifetime
/// modules will be loaded/unloaded for each request. Long lifetime modules will be kept loaded
/// until the platform is restarted.
#[derive(Debug, PartialEq, Serialize_repr, Deserialize_repr)]
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
#[derive(Debug, PartialEq, Serialize_repr, Deserialize_repr)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum UserAccount {
    Root = 0,
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
