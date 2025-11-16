# jfu Live Demo

This document shows a complete demo of all jfu features.

## Setup

```bash
cd jfu-java
```

## 1. Show Help

```bash
jfu --help
```

## 2. View Dependency Tree

```bash
jfu tree
```

Expected output:
```
⚙️ Loaded configuration from jfu.toml
📊 Dependency Tree:

📦 Main.java
  └─ Runner.java
    └─ Helper.java
  └─ Cool.java
```

## 3. Initial Build (Fresh)

```bash
jfu clean
jfu build --verbose
```

Shows all files being compiled with dependency graph.

## 4. Incremental Build (No Changes)

```bash
jfu build
```

Shows:
```
✅ Everything up to date (skipped 4 files)
```

## 5. Modify One File

```bash
echo "// Modified" >> test/Helper.java
jfu build
```

Shows only Helper.java being recompiled.

## 6. Force Rebuild

```bash
jfu build --force
```

Rebuilds everything regardless of cache.

## 7. Build and Run

```bash
jfu run
```

Builds (if needed) and runs the program.

## 8. Run with Verbose Output

```bash
jfu run --verbose
```

Shows detailed build information before running.

## 9. Configuration File

Show the configuration:

```bash
cat jfu.toml
```

The entrypoint setting allows running without specifying a file:

```bash
jfu run  # Uses entrypoint from config
```

## 10. Override Entrypoint

```bash
jfu tree Main.java  # Specific file
jfu tree            # Uses config entrypoint
```

## Performance Comparison

### Without jfu
```bash
time javac -d out test/Helper.java test/Runner.java test/Cool.java test/Main.java
time javac -d out test/Helper.java test/Runner.java test/Cool.java test/Main.java
# Both times are the same - no caching!
```

### With jfu
```bash
time jfu build  # First time: compiles all
time jfu build  # Second time: instant! ⚡
```

## Summary

jfu provides:
- ✅ Intelligent incremental compilation
- ✅ Automatic dependency resolution
- ✅ Beautiful colored output
- ✅ Simple configuration
- ✅ Multiple entry points support
- ✅ Professional CLI interface
