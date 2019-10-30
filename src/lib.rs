use chrono::Utc;
use fnv::FnvHashMap as Map;
use semver::{Version, VersionReq};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use std::fmt::{self, Display};

pub const JSONFILE: &str = "tally.json.gz";
pub const COMPFILE: &str = "computed.json.gz";

pub type DateTime = chrono::DateTime<Utc>;

#[derive(Serialize, Deserialize)]
pub struct Crate {
    pub published: Option<DateTime>,
    pub name: String,
    #[serde(rename = "vers")]
    pub version: Version,
    #[serde(rename = "deps")]
    pub dependencies: Vec<Dependency>,
    pub features: Map<String, Vec<Feature>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub req: VersionReq,
    pub features: Vec<String>,
    pub optional: bool,
    pub default_features: bool,
    #[serde(default, deserialize_with = "null_as_default")]
    pub kind: DependencyKind,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TransitiveDep {
    pub name: String,
    pub timestamp: DateTime,
    pub version: Version,
    pub transitive_count: CrateLegend,
    pub direct_count: usize,
    pub total: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DependencyKind {
    Normal,
    Build,
    Dev,
}

#[derive(Clone, Debug)]
pub enum Feature {
    Current(String),
    Dependency(String, String),
}

impl Default for DependencyKind {
    fn default() -> Self {
        DependencyKind::Normal
    }
}

fn null_as_default<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + Default,
    D: Deserializer<'de>,
{
    let option = Option::deserialize(deserializer)?;
    Ok(option.unwrap_or_default())
}

impl Display for DependencyKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            DependencyKind::Normal => "normal",
            DependencyKind::Build => "build",
            DependencyKind::Dev => "dev",
        };
        write!(f, "{}", string)
    }
}

impl Display for Feature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Feature::Current(feat) => write!(f, "{}", feat),
            Feature::Dependency(dep, feat) => write!(f, "{}/{}", dep, feat),
        }
    }
}

impl Serialize for Feature {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Feature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut s = String::deserialize(deserializer)?;
        Ok(match s.find('/') {
            Some(slash) => {
                let feature = s[slash + 1..].to_owned();
                s.truncate(slash);
                Feature::Dependency(s, feature)
            }
            None => Feature::Current(s),
        })
    }
}

mod key;
mod noop_hash;
use key::fnv_key;
use noop_hash::{PreHashedMap, PreHashedSet};
use std::iter::FromIterator;

/// Hash Set not sure if ill need it
#[derive(Debug, Serialize, Deserialize)]
pub struct CrateMapping(PreHashedSet<u64>);
impl CrateMapping {
    pub fn new() -> CrateMapping {
        Self(PreHashedSet::default())
    }
    pub fn from<I: IntoIterator<Item=u64>>(iter: I) -> CrateMapping {
         Self(PreHashedSet::from_iter(iter))
    }
    pub fn insert(&mut self, key: &str, version: &Version) {
        let k = fnv_key((&key, &version));
        self.0.insert(k);
    }
}

/// The hashmap one this will be used
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrateLegend(PreHashedMap<u64, usize>);
impl CrateLegend {
    pub fn new() -> CrateLegend {
        Self(PreHashedMap::default())
    }
    // inserts key of hashed crate name and version
    pub fn insert(&mut self, key: &str, version: &Version) {
        let k = fnv_key((&key, &version));
        *self.0.entry(k).or_insert(0) += 1;
    }

    pub fn excluded(&self, key: &str, version: &Version) -> bool {
        let k = fnv_key((&key, version));
        self.0.contains_key(&k)
    }
}
impl std::hash::Hash for CrateLegend {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.keys().for_each(|k| k.hash(state))
    }
}
