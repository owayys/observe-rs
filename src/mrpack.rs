use semver::Version;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, path::PathBuf};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DependencyId {
    Minecraft,
    Forge,
    Neoforge,
    FabricLoader,
    QuiltLoader,
    #[serde(untagged)]
    Other(String),
}

impl Display for DependencyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Minecraft => write!(f, "Minecraft"),
            Self::Forge => write!(f, "Forge"),
            Self::Neoforge => write!(f, "NeoForge"),
            Self::FabricLoader => write!(f, "Fabric"),
            Self::QuiltLoader => write!(f, "Quilt"),
            Self::Other(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHashes {
    #[serde(serialize_with = "hex::serialize")]
    #[serde(deserialize_with = "hex::deserialize")]
    pub sha1: [u8; 20],
    #[serde(serialize_with = "hex::serialize")]
    #[serde(deserialize_with = "hex::deserialize")]
    pub sha512: [u8; 64],
    #[serde(flatten)]
    #[allow(unused)]
    pub other_hashes: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub client: Requirement,
    pub server: Requirement,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Requirement {
    Required,
    Optional,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MRFile {
    pub path: PathBuf,
    pub hashes: FileHashes,
    pub env: Option<Environment>,
    pub downloads: Vec<String>,
    pub file_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MRIndex {
    pub game: String,
    pub format_version: u32,
    pub version_id: String,
    pub name: String,
    pub files: Vec<MRFile>,
    pub dependencies: HashMap<DependencyId, Version>,
}

impl Display for MRIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Name: {}, Format: {}, Version: {}",
            self.name, self.format_version, self.version_id
        )
    }
}
