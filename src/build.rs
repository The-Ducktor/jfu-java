use colored::*;
use std::{fs, path::Path, process::Command};

use crate::cache::{CacheEntry, compute_hash, load_cache, needs_rebuild, save_cache};
use crate::config::Config;
use crate::error_format::format_java_errors;
use crate::graph::{build_dependency_graph, topo_sort};

#[derive(Debug)]
pub struct BuildContext {
    pub config: Config,
    pub verbose: bool,
    pub force: bool,
}

pub fn build_files(ctx: &BuildContext, main_file: &str) -> Result<(), String> {
    // First try the current directory, then fall back to src_dir
    let main_path = if Path::new(main_file).exists() {
        main_file.into()
    } else {
        ctx.config.src_dir.join(main_file)
    };

    if !main_path.exists() {
        return Err(format!("File not found: {}", main_file));
    }

    println!("{} Checking dependencies...", "🔄".cyan());

    // Build dependency graph
    let graph = build_dependency_graph(&main_path, &ctx.config.src_dir);

    if ctx.verbose {
        println!("{} Dependency graph:", "📊".cyan());
        for (name, node) in &graph {
            println!("  {} -> {:?}", name, node.deps);
        }
    }

    // Topological sort
    let build_order = topo_sort(&graph)?;

    if ctx.verbose {
        println!("{} Build order: {:?}", "📋".cyan(), build_order);
    }

    // Create output directory
    fs::create_dir_all(&ctx.config.out_dir)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    // Load cache
    let mut cache = load_cache(&ctx.config.cache_file);

    // Determine which files need rebuilding
    let mut files_to_compile = Vec::new();
    let mut skipped = 0;

    for file_name in &build_order {
        if let Some(node) = graph.get(file_name) {
            if needs_rebuild(node, &cache, &ctx.config.out_dir, ctx.force) {
                files_to_compile.push(node.clone());
            } else {
                skipped += 1;
                if ctx.verbose {
                    println!("  {} Skipped {} (no changes)", "✓".green(), file_name);
                }
            }
        }
    }

    if files_to_compile.is_empty() {
        println!(
            "{} Everything up to date (skipped {} files)",
            "✅".green(),
            skipped
        );
        return Ok(());
    }

    // Compile files together in one javac invocation
    println!(
        "{} Compiling {} file(s)...",
        "⚡".yellow(),
        files_to_compile.len()
    );

    for node in &files_to_compile {
        if ctx.verbose {
            println!("  {} Compiling {}...", "🔨".cyan(), node.name);
        } else {
            println!("  {} {}", "⚡".yellow(), node.name);
        }
    }

    // Build javac command with all files
    let mut cmd = Command::new("javac");
    cmd.arg("-d").arg(&ctx.config.out_dir);

    for node in &files_to_compile {
        cmd.arg(&node.path);
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run javac: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Combine stdout and stderr as javac can output to both
        let error_output = if !stdout.is_empty() {
            format!("{}{}", stdout, stderr)
        } else {
            stderr.to_string()
        };

        return Err(format_java_errors(&error_output));
    }

    // Update cache for all compiled files
    for node in &files_to_compile {
        let class_name = node.name.strip_suffix(".java").unwrap_or(&node.name);
        let class_path = ctx.config.out_dir.join(format!("{}.class", class_name));

        cache.insert(
            node.name.clone(),
            CacheEntry {
                hash: compute_hash(&node.path),
                class_path: class_path.to_string_lossy().to_string(),
            },
        );
    }

    // Save cache
    save_cache(&ctx.config.cache_file, &cache);

    if skipped > 0 {
        println!(
            "{} Build complete ({} compiled, {} skipped)",
            "✅".green(),
            files_to_compile.len(),
            skipped
        );
    } else {
        println!(
            "{} Build complete ({} compiled)",
            "✅".green(),
            files_to_compile.len()
        );
    }

    Ok(())
}
