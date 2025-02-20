use std::collections::HashMap;

use git_repository as git;
use smartstring::alias::String as SmolString;
use std::fmt;
use std::hash::{Hash, Hasher};

/// A wrapper for a repository of the crates.io index.
pub struct Index {
    /// The name and path of the reference used to keep track of the last seen state of the
    /// crates.io repository. The default value is `refs/heads/crates-index-diff_last-seen`.
    pub seen_ref_name: &'static str,
    /// The name of the branch to fetch. This value also affects the tracking branch.
    pub branch_name: &'static str,
    /// The name of the symbolic name of the remote to fetch from.
    /// If `None`, obtain the remote name from the configuration of the currently checked-out branch.
    pub remote_name: Option<String>,
    /// The git repository to use for diffing
    pub(crate) repo: git::Repository,
}

/// Identify a kind of change that occurred to a crate
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Change {
    /// A crate version was added or it was unyanked.
    Added(CrateVersion),
    /// A crate version was yanked.
    Yanked(CrateVersion),
    /// The name of the crate whose file was deleted, which implies all versions were deleted as well.
    Deleted {
        /// The name of the deleted crate.
        name: String,
        /// All of its versions that were deleted along with the file.
        versions: Vec<CrateVersion>,
    },
}

impl Change {
    /// Return the added crate, if this is this kind of change.
    pub fn added(&self) -> Option<&CrateVersion> {
        match self {
            Change::Added(v) => Some(v),
            _ => None,
        }
    }

    /// Return the yanked crate, if this is this kind of change.
    pub fn yanked(&self) -> Option<&CrateVersion> {
        match self {
            Change::Yanked(v) => Some(v),
            _ => None,
        }
    }

    /// Return the deleted crate, if this is this kind of change.
    pub fn deleted(&self) -> Option<(&str, &[CrateVersion])> {
        match self {
            Change::Deleted { name, versions } => Some((name.as_str(), versions)),
            _ => None,
        }
    }
}

impl fmt::Display for Change {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Change::Added(_) => "added",
                Change::Yanked(_) => "yanked",
                Change::Deleted { .. } => "deleted",
            }
        )
    }
}

/// Section in which a dependency was defined in.
#[derive(
    Debug, Copy, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq, Hash, Ord, PartialOrd,
)]
#[serde(rename_all = "lowercase")]
pub enum DependencyKind {
    /// Used for production builds.
    Normal,
    /// Used only for tests and examples.
    Dev,
    /// Used in build scripts.
    Build,
}

/// Pack all information we know about a change made to a version of a crate.
#[derive(Default, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq, Debug)]
pub struct CrateVersion {
    /// The crate name, i.e. `clap`.
    pub name: SmolString,
    /// is the release yanked?
    pub yanked: bool,
    /// The semantic version of the crate.
    #[serde(rename = "vers")]
    pub version: SmolString,
    /// The checksum over the crate archive
    #[serde(rename = "cksum", with = "hex")]
    pub checksum: [u8; 32],
    /// All cargo features
    pub features: HashMap<String, Vec<String>>,
    /// All crate dependencies
    #[serde(rename = "deps")]
    pub dependencies: Vec<Dependency>,
}

impl CrateVersion {
    pub(crate) fn id(&self) -> u64 {
        let mut s = std::collections::hash_map::DefaultHasher::new();
        self.name.hash(&mut s);
        self.yanked.hash(&mut s);
        self.version.hash(&mut s);
        self.checksum.hash(&mut s);
        s.finish()
    }
}

/// A single dependency of a specific crate version
#[derive(
    Clone, serde::Serialize, serde::Deserialize, Ord, PartialOrd, Eq, PartialEq, Debug, Hash,
)]
pub struct Dependency {
    /// The crate name
    pub name: SmolString,
    /// The version the parent crate requires of this dependency
    #[serde(rename = "req")]
    pub required_version: SmolString,
    /// All cargo features configured by the parent crate
    pub features: Vec<String>,
    /// True if this is an optional dependency
    pub optional: bool,
    /// True if default features are enabled
    pub default_features: bool,
    /// The name of the build target
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<SmolString>,
    /// The kind of dependency, usually 'normal' or 'dev'
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<DependencyKind>,
    /// The package this crate is contained in
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<SmolString>,
}
