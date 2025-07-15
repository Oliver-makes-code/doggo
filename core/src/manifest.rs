use std::{collections::HashMap, fs, path::Path};
use serde::{Deserialize};

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplexDependencyKind {
    Workspace,
    Path(String),
    Version(String),
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize)]
pub struct ComplexDependency {
    #[serde(flatten)]
    kind: ComplexDependencyKind,
    #[serde(default)]
    features: Vec<String>
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    Simple(String),
    Complex(ComplexDependency)
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalToolchain {
    Cargo,
    Cmake,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PackageKind {
    #[serde(alias="exe")]
    #[default]
    Executable,
    #[serde(alias="staticlib")]
    #[serde(alias="static")]
    StaticLibrary,
    #[serde(alias="dynamiclib")]
    #[serde(alias="dynamic")]
    #[serde(alias="dylib")]
    #[serde(alias="so")]
    #[serde(alias="dll")]
    DynamicLibrary
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize)]
#[serde(untagged)]
pub enum HeaderPaths {
    Single(String),
    Multi(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManifestKind {
    Package {
        name: String,
        #[serde(default)]
        output: PackageKind,
        header_paths: Option<HeaderPaths>,
    },
    Workspace {
        members: Vec<String>,
    },
    ExternalLibrary {
        name: String,
        toolchain: ExternalToolchain,
        header_paths: Option<HeaderPaths>,
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Manifest {
    #[serde(flatten)]
    pub kind: ManifestKind,
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,
}

impl Manifest {
    const TOML_NAME: &'static str = "Doggo.toml";

    pub fn load(path: &str) -> Manifest {
        let full_path = Path::new(path).join(Self::TOML_NAME);
        let content = fs::read_to_string(full_path).unwrap();
        return toml::from_str(&content).unwrap();
    }
}
