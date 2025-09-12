use clap::Parser;
use doggo_core::{compiler_backend::ClangCompilerBackend, project::Workspace};

#[derive(clap_derive::Parser)]
#[command(name = "Doggo")]
#[command(about = "Bulding C/C++, without the fluff!", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    project: Option<String>,
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
}

fn main() {
    let cli = Cli::parse();

    let workspace = Workspace::load("./".into(), cli.project).unwrap().unwrap();

    let compiler = ClangCompilerBackend::new().unwrap();

    match &cli.command {
        Commands::Build { release } => {
            println!("Build");
        }

        Commands::Run { args, release } => {
            println!("Run");
        }

        Commands::GenCompileCommands => {
            println!("Gen");
        }

        Commands::Clean => {
            println!("Clean");
        }
    }
}
