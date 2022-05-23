use std::collections::HashMap;

use std::fmt;

/// Identify a kind of change that occurred to a crate
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Change {
    /// A crate version was added or it was unyanked.
    Added(CrateVersion),
    /// A crate version was yanked.
    Yanked(CrateVersion),
    /// A crate was deleted
    Deleted(String),
}

impl fmt::Display for Change {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Change::Added(_) => "added",
                Change::Yanked(_) => "yanked",
                Change::Deleted(_) => "deleted",
            }
        )
    }
}

/// Pack all information we know about a change made to a version of a crate.
#[derive(Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq, Debug)]
pub struct CrateVersion {
    /// The crate name, i.e. `clap`.
    pub name: String,
    /// is the release yanked?
    pub yanked: bool,
    /// The semantic version of the crate.
    #[serde(rename = "vers")]
    pub version: String,
    /// The checksum over the crate archive
    #[serde(rename = "cksum")]
    pub checksum: String,
    /// All cargo features
    pub features: HashMap<String, Vec<String>>,
    /// All crate dependencies
    #[serde(rename = "deps")]
    pub dependencies: Vec<Dependency>,
}

/// A single dependency of a specific crate version
#[derive(Clone, serde::Serialize, serde::Deserialize, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct Dependency {
    /// The crate name
    pub name: String,
    /// The version the parent crate requires of this dependency
    #[serde(rename = "req")]
    pub required_version: String,
    /// All cargo features configured by the parent crate
    pub features: Vec<String>,
    /// True if this is an optional dependency
    pub optional: bool,
    /// True if default features are enabled
    pub default_features: bool,
    /// The name of the build target
    pub target: Option<String>,
    /// The kind of dependency, usually 'normal' or 'dev'
    pub kind: Option<String>,
    /// The package this crate is contained in
    pub package: Option<String>,
}
