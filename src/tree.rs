use colored::*;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use crate::config::Config;
use crate::graph::{Node, build_dependency_graph};

fn print_tree(
    graph: &HashMap<String, Node>,
    root: &str,
    indent: usize,
    visited: &mut HashSet<String>,
    show_implicit: bool,
) {
    if visited.contains(root) {
        println!(
            "{}{}  {} (already shown)",
            "  ".repeat(indent),
            "└─".blue(),
            root.yellow()
        );
        return;
    }

    visited.insert(root.to_string());

    if let Some(node) = graph.get(root) {
        if indent == 0 {
            println!("{} {}", "📦".cyan(), node.name.bold().green());
        } else {
            println!(
                "{}{} {}",
                "  ".repeat(indent),
                "└─".blue(),
                node.name.green()
            );
        }

        for dep in &node.deps {
            print_tree(graph, dep, indent + 1, visited, show_implicit);
        }

        // Show implicit dependencies if enabled (only if not already in explicit deps)
        if show_implicit && !node.implicit_deps.is_empty() {
            for imp_dep in &node.implicit_deps {
                let dep_file = format!("{}.java", imp_dep);
                // Skip if this implicit dep was auto-included in explicit deps
                if node.deps.contains(&dep_file) {
                    continue;
                }
                println!(
                    "{}{}  {} {}",
                    "  ".repeat(indent + 1),
                    "└─".yellow(),
                    dep_file.magenta(),
                    "(implicit)".bright_black()
                );
                // Recursively show implicit deps if they're in the graph
                if graph.contains_key(&dep_file) {
                    print_tree(graph, &dep_file, indent + 2, visited, show_implicit);
                }
            }
        }
    }
}

pub fn show_tree(config: &Config, main_file: &str, _verbose: bool) -> Result<(), String> {
    // First try the current directory, then fall back to src_dir
    let main_path = if Path::new(main_file).exists() {
        PathBuf::from(main_file)
    } else {
        config.src_dir.join(main_file)
    };

    if !main_path.exists() {
        return Err(format!("File not found: {}", main_file));
    }

    let graph = build_dependency_graph(
        &main_path,
        &config.src_dir,
        config.auto_include_implicit_deps,
    );

    println!("{} Dependency Tree:\n", "📊".cyan());
    let mut visited = HashSet::new();
    print_tree(
        &graph,
        main_file,
        0,
        &mut visited,
        true, // Always show implicit dependencies
    );

    println!(
        "\n{} Implicit dependencies shown in {}",
        "ℹ️".cyan(),
        "magenta".magenta()
    );

    Ok(())
}
