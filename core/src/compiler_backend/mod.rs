use std::{process::Command};

use which::which;

use crate::{target_is_msvc, target_is_windows};

/// I don't know if this is any better than just keeping a string...
/// Different compilers will have different level types, but there's some commonality.
/// For now, I'll leave it as an enum so they have actual meaning behind the values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum OptLevel {
    #[default]
    Zero,
    One,
    Two,
    Three,
    Fast,
    Size,
    SizeAggressive,
}

impl OptLevel {
    pub fn string(self) -> &'static str {
        return match self {
            Self::Zero => "0",
            Self::One => "1",
            Self::Two => "2",
            Self::Three => "3",
            Self::Fast => "fast",
            Self::Size => "s",
            Self::SizeAggressive => "z",
        };
    }
}

/// For future-proofing, user-changable compiler-specific flags should be added to this struct,
/// so if we add more compiler backends in the future we'll have an easier time converting.
#[derive(Default)]
pub struct ExtraCompileOptions {
    pub opt_level: OptLevel,
    pub generate_debug: bool,
    pub lto: bool,
    pub target: String,
}

/// Compiler paths are cached here so we don't need to locate it every time we
/// try to invoke it or generate a compile command.
pub struct ClangCompilerBackend {
    compiler_path: String,
    lib_path: String,
    ar_path: String,
}

impl ClangCompilerBackend {
    pub fn new() -> which::Result<Self> {
        return Ok(Self {
            compiler_path: which("clang")?.to_str().unwrap().to_string(),
            lib_path: which("llvm-lib")?.to_str().unwrap().to_string(),
            ar_path: which("llvm-ar")?.to_str().unwrap().to_string(),
        });
    }

    pub fn get_object_suffix(&self, extra_options: &ExtraCompileOptions) -> &str {
        return if target_is_msvc(&extra_options.target) {
            ".obj"
        } else {
            ".o"
        };
    }

    pub fn get_static_suffix(&self, extra_options: &ExtraCompileOptions) -> &str {
        return if target_is_msvc(&extra_options.target) {
            ".lib"
        } else {
            ".a"
        };
    }

    pub fn get_dynamic_suffix(&self, extra_options: &ExtraCompileOptions) -> &str {
        return if target_is_windows(&extra_options.target) {
            ".dll"
        } else {
            ".so"
        };
    }

    pub fn get_library_prefix(&self, extra_options: &ExtraCompileOptions) -> &str {
        return if target_is_msvc(&extra_options.target) {
            ""
        } else {
            "lib"
        };
    }

    pub fn get_executable_suffix(&self, extra_options: &ExtraCompileOptions) -> &str {
        return if target_is_windows(&extra_options.target) {
            ".exe"
        } else {
            ""
        };
    }

    /// If gen_compile_commands is set, the command isn't run, and, instead,
    /// it returns a list of the compile commands. This is used to generate
    /// a compile_commands.json file.
    pub fn compile_object(
        &self,
        source_path: &str,
        output_path: &str,
        include_directories: &[String],
        defines: &[String],
        extra_options: &ExtraCompileOptions,
        gen_compile_commands: bool,
    ) -> std::io::Result<Option<Vec<String>>> {
        let mut args: Vec<String> = vec![];

        args.extend(["-c".into(), source_path.into()]);

        args.extend(["-o".into(), output_path.into()]);

        args.extend(include_directories.iter().map(|it| format!("-I{it}")));

        args.extend(defines.iter().map(|it| format!("-D{it}")));

        args.push(format!("-O{}", extra_options.opt_level.string()));

        if extra_options.generate_debug {
            args.push("-ggdb3".into());
        }

        args.push("-MD".into());

        args.extend(["-target".into(), extra_options.target.clone()]);

        if !target_is_msvc(&extra_options.target) {
            args.push("-fPIC".into());
        }

        if gen_compile_commands {
            let mut out_args = vec![self.compiler_path.clone()];
            out_args.extend(args);

            return Ok(Some(out_args));
        }

        let status = Command::new(&self.compiler_path).args(&args).status()?;

        if !status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Clang exited with status {}", status),
            ));
        }

        return Ok(None);
    }

    pub fn archive_objects(
        &self,
        object_paths: &[String],
        output_path: &str,
        extra_options: &ExtraCompileOptions,
    ) -> std::io::Result<()> {
        let mut args: Vec<String> = vec![];

        if target_is_msvc(&extra_options.target) {
            args.push(format!("/OUT:{}", output_path));
        } else {
            args.extend(["rcs".into(), output_path.into()]);
        }

        args.extend(object_paths.iter().cloned());

        let archiver = if target_is_msvc(&extra_options.target) {
            &self.lib_path
        } else {
            &self.ar_path
        };

        let status = Command::new(archiver).args(&args).status()?;

        if !status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Archiver exited with status {}", status),
            ));
        }

        return Ok(());
    }

    pub fn link_objects(
        &self,
        object_paths: &[String],
        output_path: &str,
        lib_paths: &[String],
        dynamic_libs: &[String],
        static_libs: &[String],
        dynamic_library: bool,
        extra_options: &ExtraCompileOptions,
    ) -> std::io::Result<()> {
        let mut args: Vec<String> = vec![];

        args.extend(object_paths.iter().cloned());

        args.extend(lib_paths.iter().map(|it| format!("-L{}", it)));
        args.extend(dynamic_libs.iter().map(|it| format!("-l{}", it)));

        if !target_is_msvc(&extra_options.target) {
            args.push(format!("-Wl,--whole-archive"));
        }

        args.extend(static_libs.iter().map(|it| format!("-l{}", it)));

        if dynamic_library {
            args.push("-shared".into());
        }

        args.extend(["-o".into(), output_path.into()]);

        args.extend(["-target".into(), extra_options.target.clone()]);

        if extra_options.lto {
            args.push("-flto".into());
        }

        let status = Command::new(&self.compiler_path).args(&args).status()?;

        if !status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Linker exited with status {}", status),
            ));
        }

        return Ok(());
    }
}
