mod archive;
mod progress;
mod setup;
mod solve;

use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "xtask", about = "LeetCode Rust workspace tooling")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Check environment and show setup recommendations")]
    Setup,
    #[command(about = "Reset solution.rs to a clean template")]
    Solve {
        #[arg(long, help = "Overwrite without confirmation")]
        force: bool,
    },
    #[command(about = "Archive current solution to archive/")]
    Archive {
        #[arg(help = "Problem name (e.g. two-sum)")]
        name: String,
        #[arg(short, long, help = "easy, medium, or hard")]
        difficulty: Option<String>,
        #[arg(short, long, help = "LeetCode tags (comma-separated)")]
        tags: Option<String>,
        #[arg(short, long, help = "Rust concepts practiced (comma-separated)")]
        rust_concepts: Option<String>,
    },
    #[command(about = "Show your solving progress")]
    Progress,
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask must be in a workspace subdirectory")
        .to_path_buf()
}

fn main() {
    let cli = Cli::parse();
    let root = workspace_root();

    let result = match cli.command {
        Command::Setup => setup::run(),
        Command::Solve { force } => solve::run(&root, force),
        Command::Archive {
            name,
            difficulty,
            tags,
            rust_concepts,
        } => archive::run(&root, &name, difficulty, tags, rust_concepts),
        Command::Progress => progress::run(&root),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
