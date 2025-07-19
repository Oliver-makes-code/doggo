use std::path::PathBuf;

use crate::{
    interner::StrReference,
    manifest::{Dependency, PackageManifest, WorkspaceManifest},
};

pub struct DependencyPartial {
    pub features: Box<[StrReference]>,
    pub name: StrReference,
    pub path: Option<StrReference>,
    pub version: Option<StrReference>,
    pub workspace: bool,
}

impl DependencyPartial {
    pub fn from(dependency: Dependency, name: StrReference) -> Self {
        return match dependency {
            Dependency::Simple(version) => Self {
                features: Box::new([]),
                name,
                version: Some(version),
                path: None,
                workspace: false,
            },

            Dependency::Complex(dependency) => Self {
                features: dependency.features.into_boxed_slice(),
                name,
                path: dependency.path,
                version: dependency.version,
                workspace: dependency.workspace,
            },
        };
    }
}

pub struct Package {
    pub name: StrReference,
    pub path: PathBuf,
    pub manifest: PackageManifest,
    pub dependencies: Box<[DependencyPartial]>,
}

pub struct Workspace {
    pub members: Box<[Package]>,
    pub manifest: WorkspaceManifest,
    pub dependencies: Box<[DependencyPartial]>,
}
