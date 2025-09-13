use serde::Deserialize;
use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

use crate::interner::StrReference;

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize)]
pub struct Dependency {
    pub path: Option<StrReference>,
    // pub vcpkg: Option<StrReference>,
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

#[derive(Debug)]
pub enum ManifestError {
    Toml(toml::de::Error),
    Io(io::Error),
}

impl From<toml::de::Error> for ManifestError {
    fn from(value: toml::de::Error) -> Self {
        return Self::Toml(value);
    }
}

impl From<io::Error> for ManifestError {
    fn from(value: io::Error) -> Self {
        return Self::Io(value);
    }
}

impl Manifest {
    const TOML_NAME: &'static str = "Doggo.toml";

    pub fn load(path: &PathBuf) -> Result<Option<Manifest>, ManifestError> {
        let path = path.join(Self::TOML_NAME);

        let full_path = Path::new(&path);

        if !full_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(full_path)?;
        return Ok(Some(toml::from_str(&content)?));
    }
}
