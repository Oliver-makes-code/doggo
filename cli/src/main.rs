use std::path::PathBuf;

use clap::Parser;
use doggo_core::{compiler_backend::ClangCompilerBackend, project::Workspace};

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

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { release, project } => {
            let workspace = Workspace::load("./".into(), project).unwrap().unwrap();

            let compiler = ClangCompilerBackend::new().unwrap();

            println!("Build {:?}", release);
        }

        Commands::Run { args, release, project } => {
            let workspace = Workspace::load("./".into(), project).unwrap().unwrap();

            let compiler = ClangCompilerBackend::new().unwrap();

            println!("Run {:?} {:?}", args, release);
        }

        Commands::GenCompileCommands => {
            let workspace = Workspace::load("./".into(), None).unwrap().unwrap();

            let compiler = ClangCompilerBackend::new().unwrap();

            println!("Gen");
        }

        Commands::Init { subcommand, path } => {
            let subcommand = subcommand.unwrap_or_default();

            println!("Init {:?} {:?}", subcommand, path);
        }
    }
}
