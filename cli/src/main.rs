use std::path::PathBuf;

use clap::Parser;
use doggo_core::{compiler_backend::ClangCompilerBackend, project::Workspace};

#[derive(clap_derive::Parser)]
#[command(name = "Doggo")]
#[command(about = "Bulding C/C++, without the fluff!", long_about = None)]
struct Cli {
    #[arg(short, long, global = true)]
    project: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(clap_derive::Subcommand)]
enum Commands {
    /// Builds the project.
    Build {
        #[arg(short, long)]
        release: bool,
    },

    /// Runs the project.
    Run {
        #[arg(short, long)]
        release: bool,
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Generates a compile_commands.json file.
    #[command(name = "idegen")]
    GenCompileCommands,

    /// Cleans up build directory.
    Clean,

    Init {
        #[arg(long, global = true)]
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

    let workspace = Workspace::load("./".into(), cli.project).unwrap().unwrap();

    let compiler = ClangCompilerBackend::new().unwrap();

    match &cli.command {
        Commands::Build { release } => {
            println!("Build {:?}", release);
        }

        Commands::Run { args, release } => {
            println!("Run {:?} {:?}", args, release);
        }

        Commands::GenCompileCommands => {
            println!("Gen");
        }

        Commands::Clean => {
            println!("Clean");
        }

        Commands::Init { subcommand, path } => {
            let subcommand = subcommand.unwrap_or_default();

            println!("Init {:?} {:?}", subcommand, path);
        }
    }
}
