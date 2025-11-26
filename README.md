

Sometimes you're working on a tiny Java project—maybe a university assignment, maybe a leetcode warm-up, maybe just messing around—and you think "I really don't want to set up a whole Maven project for this."

But you also don't want to manually run `javac` in the right order every single time.

So here we are. It's not meant to replace real build tools. It's not meant to scale. It's just meant to be _nice_ for small stuff.

## What It Does

- **Friendly error messages** (Java errors are scary, we make them less scary)


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




Run `jfu init` to get a `jfu.toml`:

⚠️ Helper.java references classes without declaring them in header:
   → Class 'HelperTest' is referenced but not declared in header
     💡 Add 'using "HelperTest.java"' to the header comment
```


📦 Main.java


1. Reads `/* using "..." */` comments from your files
2. Builds a dependency graph (DFS, topological sort, the works)
3. Hashes each file to see what changed
4. Only recompiles the changed ones
5. Runs `javac` and `java` for you

It caches everything in `.jfu/cache/` so the second build is instant. ⚡

## What It Doesn't Do

- Replace Maven/Gradle (please don't try)
- Handle complex multi-module projects
- Manage external dependencies (no JAR support yet)
- Scale to large codebases
- Make your code run faster (it just compiles faster)

## Error Messages

We made Java errors prettier because they're intimidating:

- Helpful tips


Because learning Java is hard enough without cryptic errors.



## Thanks

- Cargo (Rust's build tool) for the inspiration
- Everyone who's had to run `javac *.java` manually
- Coffee ☕

---

**Made for the vibes, not for production.**