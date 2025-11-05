use clap::{Parser, Subcommand};
use colored::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

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
// Data Structures
// ============================================================================

#[derive(Debug, Clone)]
struct Node {
    name: String,
    path: PathBuf,
    deps: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    hash: String,
    class_path: String,
}

type Cache = HashMap<String, CacheEntry>;

#[derive(Debug)]
struct BuildContext {
    config: Config,
    verbose: bool,
    force: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    #[serde(default = "default_src_dir")]
    src_dir: PathBuf,
    #[serde(default = "default_out_dir")]
    out_dir: PathBuf,
    #[serde(default = "default_cache_file")]
    cache_file: PathBuf,
    #[serde(default)]
    jvm_opts: Vec<String>,
    #[serde(default)]
    entrypoint: Option<String>,
}

fn default_src_dir() -> PathBuf {
    PathBuf::from(".")
}

fn default_out_dir() -> PathBuf {
    PathBuf::from("./out")
}

fn default_cache_file() -> PathBuf {
    PathBuf::from("./jfu-cache.json")
}

impl Default for Config {
    fn default() -> Self {
        Self {
            src_dir: default_src_dir(),
            out_dir: default_out_dir(),
            cache_file: default_cache_file(),
            jvm_opts: Vec::new(),
            entrypoint: None,
        }
    }
}

impl Config {
    fn load() -> Self {
        let config_path = PathBuf::from("jfu.toml");

        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(config) => {
                        println!("{} Loaded configuration from jfu.toml", "⚙️".cyan());
                        return config;
                    }
                    Err(e) => {
                        eprintln!("{} Failed to parse jfu.toml: {}", "⚠️".yellow(), e);
                        eprintln!("   Using default configuration");
                    }
                },
                Err(e) => {
                    eprintln!("{} Failed to read jfu.toml: {}", "⚠️".yellow(), e);
                    eprintln!("   Using default configuration");
                }
            }
        }

        Config::default()
    }
}

// ============================================================================
// Phase 1: Dependency Resolution
// ============================================================================

fn parse_dependencies(path: &Path) -> Vec<String> {
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
            if let Some(start) = line.find("dependent \"") {
                let rest = &line[start + 11..];
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

fn build_dependency_graph(main: &Path, base_dir: &Path) -> HashMap<String, Node> {
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

// ============================================================================
// Phase 2: Topological Sort
// ============================================================================

fn topo_sort(graph: &HashMap<String, Node>) -> Result<Vec<String>, String> {
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

// ============================================================================
// Phase 3: Hash-based Cache
// ============================================================================

fn load_cache(cache_path: &Path) -> Cache {
    if cache_path.exists() {
        let content = fs::read_to_string(cache_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    }
}

fn save_cache(cache_path: &Path, cache: &Cache) {
    let json = serde_json::to_string_pretty(cache).unwrap();
    fs::write(cache_path, json).unwrap_or_else(|e| {
        eprintln!("{} Failed to save cache: {}", "⚠️".yellow(), e);
    });
}

fn compute_hash(path: &Path) -> String {
    let content = fs::read(path).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}

fn needs_rebuild(node: &Node, cache: &Cache, out_dir: &Path, force: bool) -> bool {
    if force {
        return true;
    }

    let class_name = node.name.strip_suffix(".java").unwrap_or(&node.name);
    let class_path = out_dir.join(format!("{}.class", class_name));

    // If .class doesn't exist, rebuild
    if !class_path.exists() {
        return true;
    }

    // If not in cache, rebuild
    let Some(entry) = cache.get(&node.name) else {
        return true;
    };

    // If hash changed, rebuild
    let current_hash = compute_hash(&node.path);
    current_hash != entry.hash
}

// ============================================================================
// Phase 4: Build Command
// ============================================================================

fn build_files(ctx: &BuildContext, main_file: &str) -> Result<(), String> {
    let main_path = ctx.config.src_dir.join(main_file);

    if !main_path.exists() {
        return Err(format!("File not found: {}", main_path.display()));
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
        return Err(format!("Compilation failed:\n{}", stderr));
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

// ============================================================================
// Phase 5: Run Command
// ============================================================================

fn run_file(ctx: &BuildContext, main_file: &str) -> Result<(), String> {
    // First, build
    build_files(ctx, main_file)?;

    // Extract class name (Main.java -> Main)
    let class_name = main_file
        .strip_suffix(".java")
        .ok_or_else(|| format!("Invalid Java file: {}", main_file))?;

    println!("\n{} Running {}...\n", "🚀".green(), class_name);

    // Run the Java program with optional JVM opts
    let mut cmd = Command::new("java");
    cmd.arg("-cp").arg(&ctx.config.out_dir);

    // Add JVM options if specified
    for opt in &ctx.config.jvm_opts {
        cmd.arg(opt);
    }

    cmd.arg(class_name);

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run java: {}", e))?;

    // Print stdout
    print!("{}", String::from_utf8_lossy(&output.stdout));

    // Print stderr if any
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        eprint!("{}", stderr);
    }

    if !output.status.success() {
        return Err(format!("Program exited with status: {}", output.status));
    }

    Ok(())
}

// ============================================================================
// Phase 6: Clean Command
// ============================================================================

fn clean(config: &Config) -> Result<(), String> {
    let mut cleaned = Vec::new();

    if config.out_dir.exists() {
        fs::remove_dir_all(&config.out_dir)
            .map_err(|e| format!("Failed to remove output directory: {}", e))?;
        cleaned.push(config.out_dir.display().to_string());
    }

    if config.cache_file.exists() {
        fs::remove_file(&config.cache_file)
            .map_err(|e| format!("Failed to remove cache file: {}", e))?;
        cleaned.push(config.cache_file.display().to_string());
    }

    if cleaned.is_empty() {
        println!("{} Nothing to clean", "✨".cyan());
    } else {
        println!("{} Cleaned build artifacts:", "🧹".green());
        for item in cleaned {
            println!("  {} {}", "✓".green(), item);
        }
    }

    Ok(())
}

// ============================================================================
// Tree Visualization
// ============================================================================

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

fn show_tree(config: &Config, main_file: &str) -> Result<(), String> {
    let main_path = config.src_dir.join(main_file);

    if !main_path.exists() {
        return Err(format!("File not found: {}", main_path.display()));
    }

    let graph = build_dependency_graph(&main_path, &config.src_dir);

    println!("{} Dependency Tree:\n", "📊".cyan());
    let mut visited = HashSet::new();
    print_tree(&graph, main_file, 0, &mut visited);

    Ok(())
}

// ============================================================================
// Init Command
// ============================================================================

fn init_config(force: bool) -> Result<(), String> {
    let config_path = PathBuf::from("jfu.toml");

    if config_path.exists() && !force {
        return Err(format!(
            "jfu.toml already exists. Use --force to overwrite."
        ));
    }

    let template = r#"# jfu Configuration File

# Source directory containing your Java files
# Defaults to "." (current directory)
src_dir = "."

# Output directory for compiled .class files
out_dir = "./out"

# Location of the build cache file
cache_file = "./jfu-cache.json"

# Default entrypoint when no file is specified
# This is useful when you have multiple classes with main() methods
entrypoint = "Main.java"

# JVM options to pass when running your program
jvm_opts = ["-Xmx256m"]

# Future features (not yet implemented):
#
# [dependencies]
# # External JAR files to include in classpath
# libs = [
#     "lib/commons-lang3-3.12.0.jar",
# ]
#
# [compiler]
# # Additional javac options
# javac_opts = ["-Xlint:unchecked", "-g"]
"#;

    fs::write(&config_path, template).map_err(|e| format!("Failed to create jfu.toml: {}", e))?;

    println!("{} Created jfu.toml", "✅".green());
    println!("\n{}", "Configuration file created with defaults:".cyan());
    println!("  {} src_dir = \".\"", "•".blue());
    println!("  {} out_dir = \"./out\"", "•".blue());
    println!("  {} cache_file = \"./jfu-cache.json\"", "•".blue());
    println!("  {} entrypoint = \"Main.java\"", "•".blue());
    println!("  {} jvm_opts = [\"-Xmx256m\"]", "•".blue());
    println!(
        "\n{}",
        "Edit jfu.toml to customize your project settings.".cyan()
    );

    Ok(())
}

// ============================================================================
// Main Entry Point
// ============================================================================

fn main() {
    let cli = Cli::parse();

    let config = Config::load();
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
            show_tree(&config, &file)
        }
        Commands::Init { force } => init_config(force),
    };

    if let Err(e) = result {
        eprintln!("\n{} {}", "❌".red(), e.red());
        std::process::exit(1);
    }
}
