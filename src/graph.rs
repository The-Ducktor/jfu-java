use colored::*;
use regex::Regex;
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
    #[allow(dead_code)]
    pub implicit_deps: Vec<String>,
}

/// Finds all public classes in the same directory as the given file
fn find_public_classes_in_dir(file_path: &Path) -> Vec<String> {
    let mut classes = Vec::new();

    // Get the directory containing the file, or use current directory if no parent
    let dir = match file_path.parent() {
        Some(d) if !d.as_os_str().is_empty() => d,
        _ => Path::new("."),
    };

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return classes,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        // Only look at .java files
        if path.extension().and_then(|s| s.to_str()) != Some("java") {
            continue;
        }

        // Skip the current file
        if path == file_path {
            continue;
        }

        // Read the file and check if it has a public class
        if let Ok(content) = fs::read_to_string(&path) {
            // Look for public class declarations
            let class_regex = Regex::new(r"(?m)^\s*public\s+class\s+(\w+)").unwrap();

            for cap in class_regex.captures_iter(&content) {
                if let Some(class_name) = cap.get(1) {
                    let name = class_name.as_str().to_string();
                    classes.push(name);
                }
            }
        }
    }

    classes
}

/// Detects class references in the code (excluding those in comments and the header)
fn find_class_references(path: &Path, declared_deps: &[String]) -> Vec<String> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut references = HashSet::new();
    let mut in_header = true;
    let mut in_block_comment = false;
    let mut current_class_name: Option<String> = None;

    // Get the list of declared dependencies (without .java extension)
    let declared_classes: HashSet<String> = declared_deps
        .iter()
        .map(|d| d.trim_end_matches(".java").to_string())
        .collect();

    // Extract the current file's class name to exclude it from references
    let class_decl_regex = Regex::new(r"(?m)^\s*(?:public\s+)?class\s+(\w+)").unwrap();
    if let Some(cap) = class_decl_regex.captures(&content) {
        if let Some(class_name) = cap.get(1) {
            current_class_name = Some(class_name.as_str().to_string());
        }
    }

    for line in content.lines() {
        let trimmed = line.trim();

        // Track block comments
        if trimmed.starts_with("/*") {
            in_block_comment = true;
        }

        if in_block_comment {
            if trimmed.ends_with("*/") {
                in_block_comment = false;
                in_header = false; // Header block is done
            }
            continue;
        }

        // Skip single-line comments
        if trimmed.starts_with("//") {
            continue;
        }

        // If we hit actual code and haven't seen the header end, we're past the header
        if in_header
            && !trimmed.is_empty()
            && !trimmed.starts_with("//")
            && !trimmed.starts_with("/*")
        {
            in_header = false;
        }

        // Skip the header
        if in_header {
            continue;
        }

        // Look for class instantiations and references using regex
        // Matches patterns like: new ClassName(), ClassName variable, ClassName.method()
        let class_ref_regex = Regex::new(r"\b([A-Z][a-zA-Z0-9_]*)\b").unwrap();

        for cap in class_ref_regex.captures_iter(line) {
            if let Some(class_name) = cap.get(1) {
                let name = class_name.as_str().to_string();

                // Don't include if it's the current file's class
                if let Some(ref curr_class) = current_class_name {
                    if &name == curr_class {
                        continue;
                    }
                }

                // Don't include if it's already declared in dependencies
                if !declared_classes.contains(&name) {
                    references.insert(name);
                }
            }
        }
    }

    references.into_iter().collect()
}

/// Checks for implicit dependencies and returns warnings
pub fn check_implicit_dependencies(path: &Path, declared_deps: &[String]) -> Vec<String> {
    let public_classes = find_public_classes_in_dir(path);
    let referenced_classes = find_class_references(path, declared_deps);

    let mut implicit_deps = Vec::new();

    for ref_class in referenced_classes {
        // Check if this reference matches a public class in the same directory
        if public_classes.contains(&ref_class) {
            implicit_deps.push(ref_class);
        }
    }

    implicit_deps
}

pub fn parse_dependencies(path: &Path) -> (Vec<String>, Vec<String>) {
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

    // Check for implicit dependencies
    let implicit_deps = check_implicit_dependencies(path, &deps);

    (deps, implicit_deps)
}

pub fn build_dependency_graph(
    main: &Path,
    base_dir: &Path,
    auto_include_implicit: bool,
) -> HashMap<String, Node> {
    let mut visited = HashSet::new();
    let mut graph = HashMap::new();

    fn dfs(
        path: &Path,
        base: &Path,
        visited: &mut HashSet<String>,
        graph: &mut HashMap<String, Node>,
        auto_include_implicit: bool,
    ) {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if visited.contains(&name) {
            return;
        }
        visited.insert(name.clone());

        let (mut deps, implicit_deps) = parse_dependencies(path);

        // Warn about implicit dependencies
        if !implicit_deps.is_empty() {
            let file_name = path.file_name().unwrap().to_string_lossy();
            eprintln!(
                "{} {} references classes without declaring them in header:",
                "⚠️".yellow(),
                file_name.bright_white()
            );
            for imp_dep in &implicit_deps {
                eprintln!(
                    "   {} Class '{}' is referenced but not declared in header",
                    "→".yellow(),
                    imp_dep.bright_cyan()
                );
                if auto_include_implicit {
                    eprintln!(
                        "     {} Auto-including '{}.java' in compilation",
                        "✓".green(),
                        imp_dep
                    );
                } else {
                    eprintln!(
                        "     {} Add 'using \"{}.java\"' to the header comment",
                        "💡".yellow(),
                        imp_dep
                    );
                }
            }
        }

        // If auto_include_implicit is enabled, add implicit deps to explicit deps
        if auto_include_implicit {
            for imp_dep in &implicit_deps {
                let dep_file = format!("{}.java", imp_dep);
                if !deps.contains(&dep_file) {
                    deps.push(dep_file);
                }
            }
        }

        // Recursively resolve dependencies
        for dep in &deps {
            let dep_path = base.join(dep);
            if dep_path.exists() {
                dfs(&dep_path, base, visited, graph, auto_include_implicit);
            } else {
                eprintln!("{} Dependency not found: {}", "⚠️".yellow(), dep);
            }
        }

        graph.insert(
            name.clone(),
            Node {
                name,
                path: path.to_path_buf(),
                deps,
                implicit_deps,
            },
        );
    }

    dfs(
        main,
        base_dir,
        &mut visited,
        &mut graph,
        auto_include_implicit,
    );
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
