use std::{fmt::format, fs, path::PathBuf, process::exit};

use clap::Parser;
use doggo_core::{
    compiler_backend::{ClangCompilerBackend, ExtraCompileOptions, OptLevel},
    file_up_to_date, get_default_target,
    manifest::PackageKind,
    project::{Package, Workspace},
};

#[derive(clap_derive::Parser)]
#[command(name = "Doggo")]
#[command(about = "Bulding C/C++, without the fluff!", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap_derive::Subcommand)]
enum Commands {
    /// Builds the project.
    Build {
        #[arg(short, long)]
        release: bool,
        #[arg(short, long)]
        project: Option<String>,
    },

    /// Runs the project.
    Run {
        #[arg(short, long)]
        release: bool,
        #[arg(short, long)]
        project: Option<String>,
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Generates a compile_commands.json file.
    #[command(name = "idegen")]
    GenCompileCommands,

    Init {
        #[arg(long, short, global = true)]
        path: Option<PathBuf>,

        #[command(subcommand)]
        subcommand: Option<ProjectInit>,
    },
}

#[derive(clap_derive::Subcommand, Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ProjectInit {
    Dylib,
    Staticlib,
    #[default]
    Binary,
}

fn unwrap_fancy<T>(res: Result<T, Box<dyn std::error::Error>>) -> T {
    return match res {
        Err(e) => {
            eprintln!("{}", e);
            exit(0);
        }
        Ok(t) => t,
    };
}

fn compiled_path(package_name: &str) -> PathBuf {
    return PathBuf::new().join(".doggo").join(package_name);
}

fn modify_filename(
    package: &Package,
    compiler: &ClangCompilerBackend,
    extra_options: &ExtraCompileOptions,
    base_name: &str,
) -> String {
    return match package.output {
        PackageKind::Executable => format!(
            "{}.{}",
            base_name,
            compiler.get_executable_suffix(extra_options)
        ),
        PackageKind::DynamicLibrary => format!(
            "{}{}.{}",
            compiler.get_library_prefix(extra_options),
            base_name,
            compiler.get_dynamic_suffix(extra_options)
        ),
        PackageKind::StaticLibrary => format!(
            "{}{}.{}",
            compiler.get_library_prefix(extra_options),
            base_name,
            compiler.get_static_suffix(extra_options)
        ),
    };
}

fn build_package(
    workspace: &Workspace,
    package: &Package,
    compiler: &ClangCompilerBackend,
    extra_options: &ExtraCompileOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let compiled = compiled_path(&package.name.get());

    fs::create_dir_all(&compiled)?;

    let mut objects = vec![];

    package.visit(
        |path| {
            let base_path = compiled
                .join(path)
                .with_extension(compiler.get_object_suffix(extra_options));
            let output = base_path.to_str().unwrap();

            objects.push(output.to_string());

            let def_file = base_path.with_extension("d");
            let dep_file = def_file.to_str().unwrap();

            if file_up_to_date(dep_file, output)? {
                return Ok(());
            }

            compiler.compile_object(
                &package.resolve_source(path),
                output,
                &[],
                &[],
                extra_options,
                false,
            )?;

            return Ok(());
        },
        &["c", "cpp", "cxx", "c++", "cc", "s", "asm"],
    )?;

    let compiled = compiled.with_file_name(&modify_filename(
        package,
        compiler,
        extra_options,
        compiled.file_name().unwrap().to_str().unwrap(),
    ));

    let output = compiled.to_str().unwrap();

    if let PackageKind::StaticLibrary = package.output {
        compiler.archive_objects(&objects, output, extra_options)?;
    } else {
        compiler.link_objects(
            &objects,
            output,
            &[],
            &[],
            &[],
            package.output == PackageKind::DynamicLibrary,
            extra_options,
        )?;
    }

    return Ok(());
}

fn build(
    workspace: &Workspace,
    compiler: &ClangCompilerBackend,
    release: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(current_member) = workspace.current_member else {
        return Ok(());
    };

    let current_member = &workspace.members[current_member];

    let extra_options = ExtraCompileOptions {
        opt_level: if release {
            OptLevel::Three
        } else {
            OptLevel::Zero
        },
        generate_debug: true,
        lto: current_member.lto,
        target: get_default_target().to_string(),
    };

    build_package(workspace, &current_member, compiler, &extra_options)?;

    return Ok(());
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Commands::Build { release, project } => {
            let workspace = Workspace::load("./".into(), project)?.unwrap();

            let compiler = ClangCompilerBackend::new()?;

            println!("Build {:?}", release);

            build(&workspace, &compiler, release)?;
        }

        Commands::Run {
            args,
            release,
            project,
        } => {
            let workspace = Workspace::load("./".into(), project)?.unwrap();

            let compiler = ClangCompilerBackend::new()?;

            println!("Run {:?} {:?}", args, release);
        }

        Commands::GenCompileCommands => {
            let workspace = Workspace::load("./".into(), None)?.unwrap();

            let compiler = ClangCompilerBackend::new()?;

            println!("Gen");
        }

        Commands::Init { subcommand, path } => {
            let subcommand = subcommand.unwrap_or_default();

            println!("Init {:?} {:?}", subcommand, path);
        }
    }

    return Ok(());
}

fn main() {
    let cli = Cli::parse();

    unwrap_fancy(run(cli));
}
