#![feature(decl_macro, str_as_str, fn_traits)]

use std::{fs, io, path::Path, time::SystemTime};

pub mod compiler_backend;
pub mod interner;
pub mod manifest;
pub mod project;

pub const BUILD_DIR: &'static str = ".doggo";

pub fn target_is_msvc(target: &str) -> bool {
    return target.ends_with("msvc");
}

pub fn target_is_windows(target: &str) -> bool {
    return target.contains("windows");
}

pub const fn get_default_target() -> &'static str {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return "x86_64-pc-linux-gnu";

    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    return "aarch64-pc-linux-gnu";

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return "x86_64-pc-windows-msvc";

    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    return "aarch64-pc-windows-msvc";

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return "x86_64-apple-darwin";

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return "aarch64-apple-darwin";

    #[allow(unreachable_code)]
    return "unknown-unknown-unknown";
}

pub const DEFAULT_TARGET: &'static str = get_default_target();

pub fn walk_dir<F: FnMut(&str) -> io::Result<()>>(path: &Path, consumer: &mut F) -> io::Result<()> {
    let read = path.read_dir()?;

    let base_path = path.to_str().unwrap().to_string();

    for entry in read {
        let entry = entry?;

        let path = entry.path().to_str().unwrap().to_string();

        let entry_path = entry.path();
        if entry_path.is_dir() {
            walk_dir(&entry_path, consumer)?;
        } else if entry_path.is_file() {
            consumer.call_mut((&path[base_path.len() + 1..],))?;
        }
    }

    return Ok(());
}

fn read_depfile(dependency_path: &str, file_path: &str) -> std::io::Result<Vec<String>> {
    let text = fs::read_to_string(dependency_path)?;

    let depfile = depfile::parse(&text)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{}", e)))?;

    return Ok(depfile
        .find(file_path)
        .unwrap()
        .iter()
        .map(|it| it.to_string())
        .collect());
}

fn file_creation_time(path: &str) -> std::io::Result<SystemTime> {
    let metadata = fs::metadata(path)?;

    return metadata.modified();
}

pub fn file_up_to_date(dependency_path: &str, file_path: &str) -> io::Result<bool> {
    if !fs::exists(dependency_path)? || !fs::exists(file_path)? {
        return Ok(false);
    }

    let dependencies = read_depfile(dependency_path, file_path)?;

    let base_time = file_creation_time(dependency_path)?;

    for dependency in dependencies {
        if !fs::exists(&dependency)? {
            return Ok(false);
        }

        let time = file_creation_time(&dependency)?;

        if time > base_time {
            return Ok(false);
        }
    }

    return Ok(true);
}
