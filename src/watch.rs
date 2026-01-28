//! File watching functionality for auto-rebuild on changes.
//!
//! This module provides the `--watch` feature, which monitors Java source files
//! for changes and automatically triggers rebuilds.

use colored::*;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::io::Write;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use crate::build::{build_files, BuildContext};
use crate::config::Config;
use crate::run::run_file;

/// Clear the terminal screen if the flag is set.
pub fn clear_if_needed(clear: bool) {
    if clear {
        print!("\x1B[2J\x1B[1;1H");
        std::io::stdout().flush().unwrap();
    }
}

/// Watch for file changes in the source directory and auto-rebuild.
///
/// # Arguments
/// * `config` - The project configuration
/// * `file` - The main Java file to build/run
/// * `is_run` - True if this is a run command, false for build
/// * `ctx` - The build context
///
/// # Returns
/// Result indicating success or failure
pub fn watch_files(
    config: &Config,
    file: &str,
    is_run: bool,
    ctx: &BuildContext,
) -> Result<(), String> {
    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())
        .map_err(|e| format!("Failed to create file watcher: {}", e))?;

    watcher
        .watch(Path::new(&config.src_dir), RecursiveMode::Recursive)
        .map_err(|e| {
            format!(
                "Failed to watch directory {}: {}",
                config.src_dir.display(),
                e
            )
        })?;

    println!(
        "{} Watching for changes in {}...",
        "👀".green(),
        config.src_dir.display()
    );
    println!("Press Ctrl+C to stop watching.\n");

    // Initial build
    clear_if_needed(config.clear_on_run);
    let result = if is_run {
        run_file(ctx, file)
    } else {
        build_files(ctx, file)
    };

    if let Err(e) = result {
        eprintln!("\n{} {}", "❌".red(), e.red());
    }

    let mut last_rebuild = Instant::now();

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                if should_trigger_rebuild(&event) {
                    // Debounce: wait 500ms after last change
                    let now = Instant::now();
                    if now.duration_since(last_rebuild) > Duration::from_millis(500) {
                        last_rebuild = now;

                        clear_if_needed(config.clear_on_watch_rebuild);
                        println!("{} Rebuilding due to changes...", "🔄".blue());

                        let result = if is_run {
                            run_file(ctx, file)
                        } else {
                            build_files(ctx, file)
                        };

                        if let Err(e) = result {
                            eprintln!("\n{} {}", "❌".red(), e.red());
                        } else {
                            println!("\n{} Build completed successfully.", "✅".green());
                        }
                    }
                }
            }
            Ok(Err(e)) => eprintln!("Watch error: {:?}", e),
            Err(e) => eprintln!("Watch channel error: {:?}", e),
        }
    }
}

/// Check if the file event should trigger a rebuild.
///
/// Only rebuild on writes to .java files.
fn should_trigger_rebuild(event: &Event) -> bool {
    matches!(event.kind, EventKind::Modify(_))
        && event
            .paths
            .iter()
            .any(|p| p.extension().and_then(|e| e.to_str()) == Some("java"))
}
