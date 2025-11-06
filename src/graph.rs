use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub path: PathBuf,
    pub deps: Vec<String>,
}

pub fn parse_dependencies(path: &Path) -> Vec<String> {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("Failed to read file: {}", path.display()));

    let mut deps = Vec::new();
    let mut in_comment = false;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("/*") {
            in_comment = true;
        }

        if in_comment || line.starts_with("/*") {
            if let Some(start) = line.find("using \"") {
                let rest = &line[start + 7..];
                if let Some(end) = rest.find('"') {
                    let dep = &rest[..end];
                    deps.push(dep.to_string());
                }
            }
        }

        if line.ends_with("*/") {
            break; // stop after first comment block
        }

        if !in_comment && !line.starts_with("//") && !line.is_empty() {
            break; // stop after top comment block
        }
    }

    deps
}

pub fn build_dependency_graph(main: &Path, base_dir: &Path) -> HashMap<String, Node> {
    let mut visited = HashSet::new();
    let mut graph = HashMap::new();

    fn dfs(
        path: &Path,
        base: &Path,
        visited: &mut HashSet<String>,
        graph: &mut HashMap<String, Node>,
    ) {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if visited.contains(&name) {
            return;
        }
        visited.insert(name.clone());

        let deps = parse_dependencies(path);

        // Recursively resolve dependencies
        for dep in &deps {
            let dep_path = base.join(dep);
            if dep_path.exists() {
                dfs(&dep_path, base, visited, graph);
            } else {
                use colored::*;
                eprintln!("{} Dependency not found: {}", "⚠️".yellow(), dep);
            }
        }

        graph.insert(
            name.clone(),
            Node {
                name,
                path: path.to_path_buf(),
                deps,
            },
        );
    }

    dfs(main, base_dir, &mut visited, &mut graph);
    graph
}

pub fn topo_sort(graph: &HashMap<String, Node>) -> Result<Vec<String>, String> {
    let mut result = Vec::new();
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();

    fn visit(
        node_name: &str,
        graph: &HashMap<String, Node>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        result: &mut Vec<String>,
    ) -> Result<(), String> {
        if rec_stack.contains(node_name) {
            return Err(format!(
                "Circular dependency detected involving: {}",
                node_name
            ));
        }

        if visited.contains(node_name) {
            return Ok(());
        }

        rec_stack.insert(node_name.to_string());

        if let Some(node) = graph.get(node_name) {
            for dep in &node.deps {
                visit(dep, graph, visited, rec_stack, result)?;
            }
        }

        rec_stack.remove(node_name);
        visited.insert(node_name.to_string());
        result.push(node_name.to_string());

        Ok(())
    }

    // Visit all nodes
    for node_name in graph.keys() {
        visit(node_name, graph, &mut visited, &mut rec_stack, &mut result)?;
    }

    Ok(result)
}
