# jfu - Java Fast Utility

A blazingly fast, incremental build tool for Java projects inspired by Cargo. Say goodbye to slow builds and hello to intelligent caching!

## ✨ Features

- **🚀 Incremental Builds**: Only recompiles changed files using SHA-256 hashing
- **📦 Dependency Resolution**: Automatic dependency tracking via simple comments
- **🔄 Topological Sorting**: Compiles files in the correct order
- **⚡ Fast**: Skips unchanged files, saving compilation time
- **🎨 Beautiful Output**: Colored, emoji-rich terminal output
- **🧹 Clean Commands**: Easy artifact cleanup
- **📊 Dependency Trees**: Visualize your project structure
- **🔧 Simple Configuration**: Optional `jfu.toml` support (coming soon)

## 📥 Installation

### From Source

```bash
git clone <repository-url>
cd jfu-java
cargo build --release
```

The binary will be at `target/release/jfu-java`. You can copy it to your PATH:

```bash
cp target/release/jfu-java /usr/local/bin/jfu
```

## 🚀 Quick Start

### 1. Annotate Your Dependencies

Add dependency comments at the top of your Java files:

```java
/*
dependent "Helper.java"
dependent "Utils.java"
*/
public class Main {
    public static void main(String[] args) {
        Helper helper = new Helper();
        helper.doSomething();
    }
}
```

### 2. Build Your Project

```bash
jfu build Main.java
```

Output:
```
🔄 Checking dependencies...
⚡ Compiling 3 file(s)...
  ⚡ Helper.java
  ⚡ Utils.java
  ⚡ Main.java
✅ Build complete (3 compiled)
```

### 3. Run Your Code

```bash
jfu run Main.java
```

This will build (if needed) and execute your program in one command!

## 📖 Commands

### `jfu build [FILE]`

Builds the specified Java file and its dependencies.

```bash
jfu build                     # Build using entrypoint from jfu.toml (or Main.java)
jfu build App.java            # Build specific file
jfu build --verbose Main.java # Build with detailed output
jfu build --force Main.java   # Force rebuild all files
```

**Options:**
- `--verbose, -v`: Show detailed build information
- `--force, -f`: Ignore cache and rebuild everything

**Note:** If no file is specified, jfu uses the `entrypoint` from `jfu.toml`, or defaults to `Main.java`

### `jfu run [FILE]`

Builds and runs the specified Java file.

```bash
jfu run                       # Run using entrypoint from jfu.toml (or Main.java)
jfu run App.java              # Run specific file
```

Automatically:
1. Builds the project (incrementally)
2. Extracts the class name
3. Runs `java -cp out ClassName` with optional JVM options

**Note:** If no file is specified, jfu uses the `entrypoint` from `jfu.toml`, or defaults to `Main.java`

### `jfu tree [FILE]`

Displays the dependency tree for visualization.

```bash
jfu tree                      # Show tree for entrypoint from jfu.toml (or Main.java)
jfu tree Main.java            # Show tree for specific file
```

Output:
```
📊 Dependency Tree:

📦 Main.java
  └─ Runner.java
    └─ Helper.java
  └─ Cool.java
```

**Note:** If no file is specified, jfu uses the `entrypoint` from `jfu.toml`, or defaults to `Main.java`

### `jfu clean`

Removes all build artifacts.

```bash
jfu clean
```

This deletes:
- `./out/` directory (compiled classes)
- `./jfu-cache.json` (build cache)

## 🏗️ Project Structure

```
your-project/
├── Main.java          # Source files (in current directory by default)
├── Helper.java
├── Utils.java
├── jfu.toml           # Optional configuration file
├── out/               # Compiled classes (auto-generated)
│   ├── Main.class
│   ├── Helper.class
│   └── Utils.class
└── jfu-cache.json     # Build cache (auto-generated)
```

**Note:** By default, jfu looks for source files in the current directory (`.`). You can configure this via `jfu.toml`

## 🔧 How It Works

### 1. Dependency Parsing

jfu reads dependency comments from the top of each Java file:

```java
/*
dependent "Dependency1.java"
dependent "Dependency2.java"
*/
```

### 2. Dependency Graph

It builds a complete dependency graph by recursively analyzing all files.

### 3. Topological Sort

Files are sorted in compile-safe order (dependencies before dependents).

### 4. Hash-Based Caching

Each file is hashed with SHA-256. If the hash matches the cache:
- ✅ Skip compilation
- 📁 Use existing `.class` file

If the hash differs or file is new:
- ⚡ Recompile
- 💾 Update cache

### 5. Batch Compilation

Changed files are compiled together in a single `javac` invocation for proper dependency resolution.

## 📊 Cache Format

The `jfu-cache.json` stores metadata about compiled files:

```json
{
  "Main.java": {
    "hash": "a1b2c3d4...",
    "class_path": "./out/Main.class"
  },
  "Helper.java": {
    "hash": "e5f6g7h8...",
    "class_path": "./out/Helper.class"
  }
}
```

## 🎯 Use Cases

### Small to Medium Projects

Perfect for university assignments, coding challenges, or small utilities where you don't want the overhead of Maven/Gradle.

### Rapid Prototyping

Quick compilation feedback loop for testing ideas.

### Learning Java

Simple dependency management without complex build tool configuration.

### CI/CD

Fast, reproducible builds with intelligent caching.

## 🔮 Roadmap

- [x] Phase 1: Dependency Resolution
- [x] Phase 2: Topological Sort
- [x] Phase 3: Hash-based Cache
- [x] Phase 4: Build Command
- [x] Phase 5: Run Command
- [x] Phase 6: Clean Command
- [x] Phase 7: Tree Visualization
- [x] Phase 8: CLI with clap
- [x] Phase 9: Colored Output
- [x] Phase 10: Configuration File Support (`jfu.toml`)
- [x] Phase 10.1: Entrypoint Configuration
- [ ] Phase 11: Watch Mode (`jfu watch`)
- [ ] Phase 12: Automatic Dependency Discovery (scan imports)
- [ ] Phase 13: Multi-module Support
- [ ] Phase 14: JAR Packaging

## 🛠️ Configuration File

Create a `jfu.toml` in your project root for advanced configuration:

```toml
# Source directory containing your Java files
# Defaults to "." (current directory)
src_dir = "."

# Output directory for compiled .class files
out_dir = "./out"

# Location of the build cache file
cache_file = "./jfu-cache.json"

# Default entrypoint when no file is specified
# Useful when you have multiple classes with main() methods
entrypoint = "App.java"

# JVM options to pass when running your program
jvm_opts = ["-Xmx512m", "-ea"]
```

### Entrypoint Feature

The `entrypoint` setting is particularly useful for projects with multiple main classes:

```toml
entrypoint = "App.java"
```

Now you can run without specifying a file:
```bash
jfu run           # Uses App.java from config
jfu build         # Builds App.java and dependencies
jfu tree          # Shows App.java dependency tree
```

You can still override the entrypoint:
```bash
jfu run Main.java  # Runs Main.java instead
```

### Future Configuration Options

```toml
# Coming soon:
[dependencies]
# External JAR dependencies
libs = ["lib/commons-lang.jar"]
```

## 🐛 Troubleshooting

### "Compilation failed" errors

Make sure all dependencies are correctly declared in the comment block:

```java
/*
dependent "MissingFile.java"  # This file must exist!
*/
```

### Circular dependencies detected

Refactor your code to remove circular references:

```
A.java depends on B.java
B.java depends on A.java  ❌ Not allowed!
```

### Cache out of sync

Force rebuild to regenerate cache:

```bash
jfu build --force
```

Or clean and rebuild:

```bash
jfu clean
jfu build
```

## 🤝 Contributing

Contributions welcome! Areas for improvement:

- Additional commands and features
- Better error messages
- Performance optimizations
- IDE integration
- Plugin system

## 📝 License

MIT License - See LICENSE file for details

## 🙏 Acknowledgments

Inspired by:
- **Cargo** (Rust's build tool)
- **Maven/Gradle** (Java build tools)
- The need for a simple, fast Java build experience

## 📫 Support

For issues, questions, or suggestions, please open an issue on the repository.

---

**Made with ❤️ for Java developers who want fast, simple builds**