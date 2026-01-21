# jfu - Java Fast Utility

A friendly, incremental build tool for small Java projects. No Maven. No Gradle. Just compile what you changed.

Sometimes you're working on a tiny Java project—maybe a university assignment, maybe a LeetCode warm-up, maybe just messing around—and you think "I really don't want to set up a whole Maven project for this."

But you also don't want to manually run `javac` in the right order every single time.

So here we are. It's not meant to replace real build tools. It's not meant to scale. It's just meant to be _nice_ for small stuff.

## What It Does

- **Incremental compilation** - Only recompiles files you changed
- **Dependency tracking** - Automatically builds files in the right order
- **Smart caching** - Second builds are instant ⚡
- **Beautiful error messages** - Java errors are less scary
- **Interactive search** - Quick lookups in the Java standard library
- **JAR support** - Use external libraries with glob pattern classpaths

## Installation

```bash
# Build it (you need Rust)
cargo build --release

# Or run it directly
cargo run --release -- run YourFile.java
```

## Quick Start

### 1. Mark your dependencies

Add a comment at the top of your Java file:

```java
/* using "Helper.java" */

public class Main {
    public static void main(String[] args) {
        Helper.doSomething();
    }
}
```

### 2. Build and run

```bash
jfu run Main.java
```

That's it. It figures out the rest.

## Commands

- `jfu build [file]` - Compiles the file and dependencies
- `jfu run [file]` - Compiles and runs the file
- `jfu tree [file]` - Show the dependency tree
- `jfu clean` - Remove build artifacts
- `jfu init [--force]` - Create a `jfu.toml` config file
- `jfu search [class/method]` - Search Java documentation
- `jfu search -i` - Interactive search mode

## Global Flags

- `-v, --verbose` - Print detailed build information
- `-f, --force` - Force rebuild, ignore cache
- `--auto-implicit` - Automatically include all public classes in src_dir

## Configuration (jfu.toml)

jfu reads optional configuration from `jfu.toml`:

```toml
# Source directory containing .java files
src_dir = "."

# Output directory for compiled .class files
out_dir = "./out"

# Default entry point when none is specified
entrypoint = "Main"

# JVM options (passed to `java` command)
jvm_opts = ["-Xmx1g"]

# Automatically include implicit dependencies from same directory
auto_include_implicit_deps = false

# External JAR files (supports glob patterns including **)
classpath = [
    "libs/*.jar",
    "vendor/**/lib.jar"  # Recursive patterns work too
]
```

## How It Works

1. **Reads dependency declarations** - Parses `/* using "..." */` comments
2. **Builds a dependency graph** - Uses DFS and topological sort
3. **Checks the cache** - Hashes each file to detect changes
4. **Compiles efficiently** - Only rebuilds changed files
5. **Handles errors beautifully** - Formats Java errors for readability

The cache is stored in `.jfu/` so the second build is nearly instant.

## What It Doesn't Do

- Replace Maven/Gradle for real projects
- Handle complex multi-module projects
- Manage dependency versions (you download JARs yourself)
- Scale to large codebases
- Make your code run faster (it just compiles faster)

## Troubleshooting

### "File not found" errors

**Problem**: `jfu` can't find your Java file

**Solutions**:
- Check that the filename matches exactly: `jfu run Main.java` (case-sensitive)
- Verify the file exists: `ls Main.java`
- If using non-standard directories, set `src_dir` in `jfu.toml`

### Missing dependencies

**Problem**: Build fails with "cannot find symbol" even though you have the file

**Solutions**:
- Add dependency declaration at the top of your file:
  ```java
  /* using "Helper.java" */
  ```
- Check for typos in filenames (case-sensitive on Linux/Mac)
- If files are in subdirectories, use `src_dir` in `jfu.toml`
- If you get a warning about implicit dependencies:
  ```
  ⚠️ Helper.java references classes without declaring them in header
  ```
  Add the missing file to the `using` comment

### External JAR files not found

**Problem**: `java.lang.ClassNotFoundException` or classes from JARs don't compile

**Solutions**:
- Add to `jfu.toml`:
  ```toml
  classpath = ["libs/*.jar"]
  ```
- Download JARs into the `libs/` directory
- Use glob patterns for flexibility: `vendor/**/lib*.jar`
- If it's a build-time dependency (compiling against it), it must be on the compile classpath
- If it's a runtime dependency, it must be on the runtime classpath

### Scanner/Input issues

**Problem**: Program hangs when trying to read input with `Scanner`

**Solution**: This is usually an issue with how jfu pipes I/O. Try:
- Make sure your Scanner reads from `System.in`:
  ```java
  Scanner sc = new Scanner(System.in);
  ```
- If it still doesn't work, file an issue on GitHub

### Pretty error messages not showing

**Problem**: You see raw Java compiler output instead of formatted errors

**Cause**: This shouldn't happen in normal use

**Solution**:
- Check your terminal supports colored output: `echo $TERM`
- Update to the latest version of jfu
- If still broken, file an issue

### Cache issues

**Problem**: Build doesn't pick up your changes, or fails unexpectedly

**Solutions**:
- Clear the cache: `jfu clean`
- Force rebuild: `jfu -f run Main.java`
- Check if files have unexpected permissions: `ls -la`

## Error Message Examples

### Compilation Error

```
💥 Compilation Failed

Error #1 ───────────────────────────────────────────────
  📄 ./test/Main.java
  📍 Line 10
  💬 cannot find symbol: method doSomething()

  doSomething();
  ^

  💡 Did you mean:
    → Instead of Helper.doSomething(), try:
      • public static void doSomething()

───────────────────────────────────────────────────────────
📊 1 error

💡 Fix the errors above and try again.
```

### Runtime Error (with stack trace)

```
💥 Runtime Error
─────────────────────────────────────────────────────────

  🔥 java.lang.NullPointerException: Cannot invoke method on null
    → at MyClass.process(MyClass.java:25)
    → at MyClass.main(MyClass.java:10)
    · at java.base/java.lang.reflect.Method.invoke(Method.java:580)

─────────────────────────────────────────────────────────
💡 Check the stack trace above to find the issue.
```

## Contributing

Found a bug? Have an idea? Check out the source on GitHub and submit an issue or PR.

## Credits

- Inspired by Cargo, Rust's excellent build tool
- Every student who's had to run `javac *.java` manually
- Coffee ☕

---

**Made for the vibes, not for production.**
