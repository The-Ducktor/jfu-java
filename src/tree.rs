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
            print_tree(graph, dep, indent + 1, visited);
        }
    }
}

pub fn show_tree(config: &Config, main_file: &str) -> Result<(), String> {
    // First try the current directory, then fall back to src_dir
    let main_path = if Path::new(main_file).exists() {
        PathBuf::from(main_file)
    } else {
        config.src_dir.join(main_file)
    };

    if !main_path.exists() {
        return Err(format!("File not found: {}", main_file));
    }

    let graph = build_dependency_graph(&main_path, &config.src_dir);

    println!("{} Dependency Tree:\n", "📊".cyan());
    let mut visited = HashSet::new();
    print_tree(&graph, main_file, 0, &mut visited);

    Ok(())
}
