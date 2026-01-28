//! Build orchestration for Java files with incremental compilation.
//!
//! This module handles the core build process:
//! 1. Resolving the main file path
//! 2. Building a dependency graph from `/* using "..." */` comments
//! 3. Topologically sorting files to determine compilation order
//! 4. Checking the cache to skip unchanged files
//! 5. Running javac on files that need recompilation
//! 6. Updating the cache with new file hashes

use colored::*;
use std::{fs, path::Path, process::Command};

use crate::cache::{compute_hash, load_cache, needs_rebuild, save_cache, CacheEntry};
use crate::classpath::{build_classpath_string, resolve_classpath};
use crate::config::Config;
use crate::error_format::format_java_errors;
use crate::graph::{build_dependency_graph, topo_sort};

/// Build context containing configuration, verbose flag, and force rebuild flag.
///
/// This is passed to the build process to control compilation behavior.
#[derive(Debug)]
pub struct BuildContext {
    /// Project configuration (paths, JVM options, etc.)
    pub config: Config,
    /// If true, print detailed build information
    pub verbose: bool,
    /// If true, rebuild all files regardless of cache status
    pub force: bool,
}

/// Build a Java file and all its dependencies with incremental compilation.
///
/// This function:
/// - Locates the main Java file in the current directory or configured source directory
/// - Extracts dependencies from `/* using "..." */` comments
/// - Builds a dependency graph and topologically sorts files
/// - Skips unchanged files using xxHash64 hashing
/// - Resolves classpath globs to find external JARs
/// - Invokes javac with proper classpath and error formatting
/// - Updates the build cache with new file hashes
///
/// # Arguments
/// * `ctx` - Build context with configuration and flags
/// * `main_file` - Name of the main Java file (e.g., "Main.java")
///
/// # Errors
/// Returns an error if the file is not found, compilation fails, or cache operations fail.
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

    // No message here - will show in compilation phase

    // Build dependency graph
    let graph = build_dependency_graph(
        &main_path,
        &ctx.config.src_dir,
        ctx.config.auto_include_implicit_deps,
    );

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
            "    {} {} class file(s) ({} up-to-date)",
            "Finished".green().bold(),
            skipped,
            skipped
        );
        return Ok(());
    }

    // Compile files together in one javac invocation
    println!(
        "   {} {} file(s)",
        "Compiling".green().bold(),
        files_to_compile.len()
    );

    // Resolve classpath from glob patterns and build the classpath string
    let resolved_jars = resolve_classpath(&ctx.config.classpath);
    let classpath = build_classpath_string(&resolved_jars, &ctx.config.out_dir);

    if ctx.verbose && !resolved_jars.is_empty() {
        println!(
            "{} Resolved {} JAR file(s)",
            "📦".cyan(),
            resolved_jars.len()
        );
    }

    // Build javac command with all files
    let mut cmd = Command::new("javac");
    cmd.arg("-d").arg(&ctx.config.out_dir);

    // Add classpath if there are JARs or we need to include the output directory
    if !resolved_jars.is_empty() || !ctx.config.classpath.is_empty() {
        cmd.arg("-cp").arg(&classpath);
    }

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
            "    {} {} class file(s) ({} compiled, {} up-to-date)",
            "Finished".green().bold(),
            files_to_compile.len() + skipped,
            files_to_compile.len(),
            skipped
        );
    } else {
        println!(
            "    {} {} class file(s) ({} compiled)",
            "Finished".green().bold(),
            files_to_compile.len(),
            files_to_compile.len()
        );
    }

    Ok(())
}
