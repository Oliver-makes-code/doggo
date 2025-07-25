use std::error::Error;

use crate::{
    interner::{StrReference, StringPool},
    manifest::{Dependency, Manifest, ManifestKind, PackageKind},
};

pub struct Package {
    pub name: StrReference,
    pub path: StrReference,
    pub dependencies: im::HashMap<StrReference, Dependency>,
    pub output: PackageKind,
    pub lto: bool,
}

pub struct Workspace {
    pub members: Box<[Package]>,
    pub path: StrReference,
    pub dependencies: im::HashMap<StrReference, Dependency>,
}

impl Package {
    pub fn load(path: &str) -> Result<Option<Self>, Box<dyn Error>> {
        let manifest = Manifest::load(path)?;

        return Self::from_manifest(StringPool::global().acquire(path.into())?, manifest);
    }

    fn from_manifest(path: StrReference, manifest: Manifest) -> Result<Option<Self>, Box<dyn Error>> {
        let ManifestKind::Package(package) = manifest.kind else {
            return Ok(None);
        };

        return Ok(Some(Self {
            name: package.name,
            path,
            dependencies: manifest.dependencies.into(),
            output: package.output,
            lto: package.lto,
        }));
    }
}

impl Workspace {
}
