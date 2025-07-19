#![feature(decl_macro, str_as_str)]

pub mod compiler_backend;
pub mod interner;
pub mod manifest;
pub mod project;

pub fn target_is_msvc(target: &str) -> bool {
    return target.ends_with("msvc");
}

pub fn target_is_windows(target: &str) -> bool {
    return target.contains("windows");
}

const fn get_default_target() -> &'static str {
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
