use std::{collections::HashMap, ffi::OsStr, io, ops::Deref, path::PathBuf};


use crate::{
    interner::StrReference,
    manifest::{Dependency, Manifest, ManifestError, ManifestKind, PackageKind, WorkspaceManifest},
    walk_dir,
};

#[derive(Debug)]
pub struct Package {
    pub name: StrReference,
    pub path: PathBuf,
    pub dependencies: im::HashMap<StrReference, Dependency>,
    pub output: PackageKind,
    pub lto: bool,
}

#[derive(Debug)]
pub struct Workspace {
    pub members: Box<[Package]>,
    pub current_member: Option<usize>,
    pub path: PathBuf,
    pub dependencies: im::HashMap<StrReference, Dependency>,
}

#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
    #[error("Failed to parse TOML manifest: {0}")]
    Toml(toml::de::Error),
    #[error("IO error: {0}")]
    Io(io::Error),
    #[error("Package not found ({0})")]
    PackageNotFound(PathBuf),
    #[error("Expected a package manifest but found a workspace manifest ({0})")]
    ExpectedPackage(PathBuf),
    #[error("Package from CWD ({0}) is inside workspace ({1}), but isn't listed as a member")]
    PackageNotInWorkspace(PathBuf, PathBuf),
    #[error("Cannot find member {0} in workspace ({1}).")]
    CannotFindMember(String, PathBuf),
    #[error("Dependency named {0} in workspace ({1}) is invalid.")]
    InvalidDependency(String, PathBuf),
}

impl Package {
    pub fn load(path: &PathBuf) -> Result<Option<Self>, WorkspaceError> {
        let Some(manifest) = Manifest::load(path)? else {
            return Ok(None);
        };

        return Self::from_manifest(path, manifest);
    }

    fn from_manifest(path: &PathBuf, manifest: Manifest) -> Result<Option<Self>, WorkspaceError> {
        let ManifestKind::Package(package) = manifest.kind else {
            return Err(WorkspaceError::ExpectedPackage(path.clone()));
        };

        return Ok(Some(Self {
            name: package.name,
            path: path.canonicalize()?,
            dependencies: manifest.dependencies.into(),
            output: package.output,
            lto: package.lto,
        }));
    }

    pub fn resolve_source(&self, name: &str) -> String {
        return self.path.join("src").join(name).to_str().unwrap().to_string();
    }

    pub fn visit<F: FnMut(&str) -> io::Result<()> + Copy>(
        &self,
        mut consumer: F,
        exts: &[&str],
    ) -> io::Result<()> {
        return walk_dir(&self.path.join("src"), move |file: &str| {
            let path: PathBuf = file.into();

            let Some(ext) = path.extension().and_then(OsStr::to_str) else {
                return Ok(());
            };

            if exts.contains(&ext.to_lowercase().deref()) {
                consumer(file)?;
            }

            return Ok(());
        });
    }
}

impl From<toml::de::Error> for WorkspaceError {
    fn from(value: toml::de::Error) -> Self {
        return Self::Toml(value);
    }
}

impl From<io::Error> for WorkspaceError {
    fn from(value: io::Error) -> Self {
        return Self::Io(value);
    }
}

impl From<ManifestError> for WorkspaceError {
    fn from(value: ManifestError) -> Self {
        return match value {
            ManifestError::Io(io) => io.into(),
            ManifestError::Toml(toml) => toml.into(),
        };
    }
}

impl Workspace {
    fn from(
        manifest: WorkspaceManifest,
        dependencies: HashMap<StrReference, Dependency>,
        path: PathBuf,
    ) -> Result<Self, WorkspaceError> {
        let mut packages = vec![];

        for package in manifest.members {
            let mut package_path = path.clone();

            package_path = package_path.join(&*package.get());

            let Some(package) = Package::load(&package_path)? else {
                return Err(WorkspaceError::PackageNotFound(package_path));
            };

            packages.push(package);
        }

        return Ok(Self {
            path: path.canonicalize()?,
            dependencies: dependencies.into(),
            members: packages.into_boxed_slice(),
            current_member: None,
        });
    }

    fn find_first_workspace(mut path: PathBuf) -> Result<Option<Self>, WorkspaceError> {
        loop {
            let manifest = Manifest::load(&path)?;

            if let Some(manifest) = manifest
                && let ManifestKind::Workspace(ws) = manifest.kind
            {
                return Ok(Some(Self::from(ws, manifest.dependencies, path)?));
            }

            if !path.pop() {
                break;
            }
        }

        return Ok(None);
    }

    fn find_first_package(mut path: PathBuf) -> Result<Option<Self>, WorkspaceError> {
        loop {
            if let Some(package) = Package::load(&path)? {
                for dependency in &package.dependencies {
                    if dependency.1.workspace {
                        return Err(WorkspaceError::InvalidDependency(
                            dependency.0.get().to_string(),
                            package.path,
                        ));
                    }
                }

                return Ok(Some(Self {
                    path: path.canonicalize()?,
                    dependencies: package.dependencies.clone(),
                    members: [package].into(),
                    current_member: Some(0),
                }));
            }

            if !path.pop() {
                break;
            }
        }

        return Ok(None);
    }

    fn map_expected_package_to_none(
        value: Result<Option<Self>, WorkspaceError>,
    ) -> Result<Option<Self>, WorkspaceError> {
        if let Err(WorkspaceError::ExpectedPackage(_)) = value {
            return Ok(None);
        }

        return value;
    }

    pub fn load(
        path: PathBuf,
        selected_project: Option<String>,
    ) -> Result<Option<Self>, WorkspaceError> {
        if let Some(mut workspace) = Self::find_first_workspace(path.clone())? {
            let mut member = None;

            for dependency in &workspace.dependencies {
                if dependency.1.workspace {
                    return Err(WorkspaceError::InvalidDependency(
                        dependency.0.get().to_string(),
                        workspace.path,
                    ));
                }
            }

            if let Some(selected_project) = selected_project {
                for (index, package) in workspace.members.iter().enumerate() {
                    if &package.name.get().deref() == &selected_project.deref() {
                        member = Some(index);
                        break;
                    }
                }

                if member.is_none() {
                    return Err(WorkspaceError::CannotFindMember(
                        selected_project,
                        workspace.path,
                    ));
                }

                workspace.current_member = member;

                return Ok(Some(workspace));
            }

            for (index, package) in workspace.members.iter().enumerate() {
                if path.starts_with(&package.path) {
                    member = Some(index);
                    break;
                }
            }

            if member.is_none()
                && let Some(package) =
                    Self::map_expected_package_to_none(Self::find_first_package(path))?
                && package.path.starts_with(&workspace.path)
            {
                return Err(WorkspaceError::PackageNotInWorkspace(
                    package.path.clone(),
                    workspace.path.clone(),
                ));
            }

            workspace.current_member = member;

            return Ok(Some(workspace));
        }

        return Self::find_first_package(path);
    }
}
