# jfu - Java Fast Utility

> A quick, fun little build tool for Java when you just can't be bothered with Maven/Gradle.

Vibe-coded in Rust for those times when you just want to `javac` some files but with *slightly* more intelligence. That's it. That's the tool.

## Why Does This Exist?

Sometimes you're working on a tiny Java project—maybe a university assignment, maybe a leetcode warm-up, maybe just messing around—and you think "I really don't want to set up a whole Maven project for this."

But you also don't want to manually run `javac` in the right order every single time.

So here we are. It's not meant to replace real build tools. It's not meant to scale. It's just meant to be _nice_ for small stuff.

## What It Does

- **Tracks dependencies** via simple comments at the top of your Java files
- **Only recompiles what changed** (SHA-256 hashing, because why not)
- **Pretty output** with colors and emojis (we're not animals)
- **Friendly error messages** (Java errors are scary, we make them less scary)
- **Dead simple** to use

That's literally it.

## Quick Start

```bash
# Build it (you need Rust)
cargo build --release

# Or just run it
cargo run --release -- run YourFile.java
```

### Using It

Add a comment at the top of your Java file:

```java
/*
using "Helper.java"
using "Utils.java"
*/
public class Main {
    public static void main(String[] args) {
        Helper.doThing();
    }
}
```

Then:

```bash
jfu run Main.java
```

Done. It'll figure out the rest.

## Commands

- `jfu init` - Makes a config file (optional, but nice)
- `jfu build [file]` - Compiles stuff
- `jfu run [file]` - Compiles and runs stuff
- `jfu clean` - Deletes the `out/` folder
- `jfu tree [file]` - Shows your dependency tree (it's pretty)

Add `--verbose` or `--force` if you're feeling fancy.

## Configuration (Optional)

Run `jfu init` to get a `jfu.toml`:

```toml
src_dir = "."              # Where your .java files live
out_dir = "./out"          # Where .class files go
entrypoint = "Main.java"   # Default file to run
jvm_opts = ["-Xmx256m"]    # JVM flags
```

Now you can just type `jfu run` without specifying a file. Neat.

## How It Works

1. Reads `/* using "..." */` comments from your files
2. Builds a dependency graph (DFS, topological sort, the works)
3. Hashes each file to see what changed
4. Only recompiles the changed ones
5. Runs `javac` and `java` for you

It caches everything in `jfu-cache.json` so the second build is instant. ⚡

## What It Doesn't Do

- Replace Maven/Gradle (please don't try)
- Handle complex multi-module projects
- Manage external dependencies (no JAR support yet)
- Scale to large codebases
- Make your code run faster (it just compiles faster)

## Error Messages

We made Java errors prettier because they're intimidating:

**Compilation errors** get formatted nicely with:
- 📄 File and line number highlighted
- 💬 Clear error message
- Color-coded context
- Helpful tips

**Runtime errors** (exceptions) show:
- 🔥 Exception type in bold
- 📍 Your code highlighted in cyan
- Stack trace formatted clearly

**Recursion errors** (StackOverflow) get special treatment:
- 🔄 Clear "infinite recursion" warning
- 💡 Common causes listed
- Shows where the recursion is happening

Because learning Java is hard enough without cryptic errors.

## Vibe Check

This is a weekend project that got out of hand. It's not production-ready. It's not enterprise-grade. It's just a fun little tool that makes compiling small Java projects less annoying.

If it works for you, awesome! If you want more features, fork it or just use a real build tool. No pressure.

## License

MIT - Do whatever you want with it.

## Thanks

- Cargo (Rust's build tool) for the inspiration
- Everyone who's had to run `javac *.java` manually
- Coffee ☕

---

**Made for the vibes, not for production.**