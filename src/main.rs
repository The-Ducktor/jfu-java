//! # jfu - A Fast, Incremental Build Tool for Java
//!
//! jfu (Java Fast Utility) is designed for small Java projects where full build systems like Maven
//! or Gradle would be overkill. It provides:
//!
//! - **Incremental compilation**: Only recompiles files that have changed
//! - **Dependency management**: Automatically builds files in the correct order
//! - **Beautiful error messages**: Makes Java compilation errors less intimidating
//! - **Fast rebuilds**: Second builds are nearly instant thanks to smart caching
//!
//! ## Usage
//!
//! Add dependency declarations at the top of your Java files:
//!
//! ```java
//! /* using "Helper.java" */
//! ```
//!
//! Then run:
//! ```bash
//! jfu run Main.java
//! ```
//!
//! ## Note
//!
//! This is designed for university assignments, coding practice, and small projects.
//! For production use, please use Maven or Gradle.

use clap::{Parser, Subcommand};
use colored::*;

mod build;
mod cache;
mod classpath;
mod clean;
mod config;
mod docs;
mod error_format;
mod fuzzy;
mod graph;
mod init;
mod run;
mod search;
mod syntax;
mod tree;

use build::{BuildContext, build_files};
use clean::clean;
use config::Config;
use docs::init_docs;
use init::init_config;
use run::run_file;
use search::{interactive_search, search_class, search_methods};
use tree::show_tree;

// ============================================================================
// CLI Definition
// ============================================================================

/// jfu CLI command structure with global flags
#[derive(Parser)]
#[command(name = "jfu")]
#[command(about = "A fast, incremental build tool for Java", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output for debugging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Force rebuild, ignoring the cache
    #[arg(short, long, global = true)]
    force: bool,

    /// Automatically include all public classes in the same directory as implicit dependencies
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
    /// Search Java documentation
    Search {
        /// Class name and optional method query
        args: Vec<String>,

        /// Start interactive search mode
        #[arg(short, long)]
        interactive: bool,
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
        Commands::Search { args, interactive } => {
            // Initialize docs with verbose flag if needed
            init_docs(cli.verbose);

            if interactive {
                interactive_search()
            } else if args.is_empty() {
                Err("Please provide a class name to search or use --interactive".to_string())
            } else {
                let class = &args[0];
                if args.len() == 1 {
                    search_class(class, cli.verbose)
                } else {
                    let method_query = args[1..].join(" ");
                    search_methods(class, Some(&method_query))
                }
            }
        }
    };

    if let Err(e) = result {
        eprintln!("\n{} {}", "❌".red(), e.red());
        std::process::exit(1);
    }
}
