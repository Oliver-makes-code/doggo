use serde::Deserialize;
use std::{collections::HashMap, error::Error, fs, path::Path};

use crate::interner::StrReference;

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize)]
pub struct Dependency {
    pub path: Option<StrReference>,
    #[serde(default)]
    pub workspace: bool,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PackageKind {
    #[serde(alias = "exe")]
    #[default]
    Executable,
    #[serde(alias = "staticlib")]
    #[serde(alias = "static")]
    StaticLibrary,
    #[serde(alias = "dynamiclib")]
    #[serde(alias = "dynamic")]
    #[serde(alias = "dylib")]
    #[serde(alias = "so")]
    #[serde(alias = "dll")]
    DynamicLibrary,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize)]
#[serde(untagged)]
pub enum HeaderPaths {
    Single(StrReference),
    Multi(Vec<StrReference>),
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize)]
pub struct PackageManifest {
    pub name: StrReference,
    #[serde(default)]
    pub output: PackageKind,
    #[serde(default)]
    pub lto: bool,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize)]
pub struct WorkspaceManifest {
    #[serde(default)]
    pub members: Vec<StrReference>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManifestKind {
    Package(PackageManifest),
    Workspace(WorkspaceManifest),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Manifest {
    #[serde(flatten)]
    pub kind: ManifestKind,
    #[serde(default)]
    pub dependencies: HashMap<StrReference, Dependency>,
}

impl Manifest {
    const TOML_NAME: &'static str = "Doggo.toml";

    pub fn load(path: &str) -> Result<Manifest, Box<dyn Error>> {
        let full_path = Path::new(path).join(Self::TOML_NAME);
        let content = fs::read_to_string(full_path)?;
        return Ok(toml::from_str(&content)?);
    }
}
