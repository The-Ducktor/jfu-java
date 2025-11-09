use clap::{Parser, Subcommand};
use colored::*;

mod build;
mod cache;
mod clean;
mod config;
mod error_format;
mod graph;
mod init;
mod run;
mod syntax;
mod tree;

use build::{BuildContext, build_files};
use clean::clean;
use config::Config;
use init::init_config;
use run::run_file;
use tree::show_tree;

// ============================================================================
// CLI Definition
// ============================================================================

#[derive(Parser)]
#[command(name = "jfu")]
#[command(about = "A fast, incremental build tool for Java", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Force rebuild (ignore cache)
    #[arg(short, long, global = true)]
    force: bool,

    /// Automatically include implicit dependencies in compilation
    #[arg(long, global = true)]
    auto_implicit: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the specified Java file and its dependencies
    Build {
        /// Main Java file to build (uses entrypoint from jfu.toml or Main.java if not specified)
        file: Option<String>,
    },
    /// Build and run the specified Java file
    Run {
        /// Main Java file to run (uses entrypoint from jfu.toml or Main.java if not specified)
        file: Option<String>,
    },
    /// Clean build artifacts
    Clean,
    /// Show dependency tree
    Tree {
        /// Main Java file to analyze (uses entrypoint from jfu.toml or Main.java if not specified)
        file: Option<String>,
    },
    /// Initialize a new jfu.toml configuration file
    Init {
        /// Overwrite existing jfu.toml if present
        #[arg(long)]
        force: bool,
    },
}

// ============================================================================
// Main Entry Point
// ============================================================================

fn main() {
    let cli = Cli::parse();

    let mut config = Config::load();

    // CLI flag overrides config file
    if cli.auto_implicit {
        config.auto_include_implicit_deps = true;
    }

    let ctx = BuildContext {
        config: config.clone(),
        verbose: cli.verbose,
        force: cli.force,
    };

    let result = match cli.command {
        Commands::Build { file } => {
            let file = file
                .or_else(|| config.entrypoint.clone())
                .unwrap_or_else(|| "Main.java".to_string());
            build_files(&ctx, &file)
        }
        Commands::Run { file } => {
            let file = file
                .or_else(|| config.entrypoint.clone())
                .unwrap_or_else(|| "Main.java".to_string());
            run_file(&ctx, &file)
        }
        Commands::Clean => clean(&config),
        Commands::Tree { file } => {
            let file = file
                .or_else(|| config.entrypoint.clone())
                .unwrap_or_else(|| "Main.java".to_string());
            show_tree(&config, &file, cli.verbose)
        }
        Commands::Init { force } => init_config(force),
    };

    if let Err(e) = result {
        eprintln!("\n{} {}", "❌".red(), e.red());
        std::process::exit(1);
    }
}
