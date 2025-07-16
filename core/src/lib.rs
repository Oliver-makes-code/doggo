#![feature(decl_macro)]

pub mod compiler_backend;
pub mod manifest;

pub fn target_is_msvc(target: &str) -> bool {
    return target.ends_with("msvc");
}

pub fn target_is_windows(target: &str) -> bool {
    return target.contains("windows");
}
