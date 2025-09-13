#![feature(decl_macro, str_as_str)]

use std::{io, path::Path};

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

pub fn walk_dir<F: FnMut(&str) -> io::Result<()> + Copy>(
    path: &Path,
    mut consumer: F,
) -> io::Result<()> {
    let read = path.read_dir()?;

    let base_path = path.to_str().unwrap().to_string();

    for entry in read {
        let entry = entry?;

        let path = entry.path().to_str().unwrap().to_string();

        let entry_path = entry.path();
        if entry_path.is_dir() {
            walk_dir(&entry_path, consumer)?;
        } else if entry_path.is_file() {
            consumer(&path[base_path.len()+1..])?;
        }
    }

    return Ok(());
}
