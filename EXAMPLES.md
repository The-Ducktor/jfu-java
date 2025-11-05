# jfu Examples

This guide provides practical examples of using jfu for various Java projects.

## 📚 Table of Contents

1. [Simple Hello World](#simple-hello-world)
2. [Project with Dependencies](#project-with-dependencies)
3. [Multiple Classes](#multiple-classes)
4. [Incremental Build Demo](#incremental-build-demo)
5. [Using Configuration File](#using-configuration-file)
6. [Advanced Patterns](#advanced-patterns)

---

## Simple Hello World

The simplest possible jfu project:

### HelloWorld.java
```java
public class HelloWorld {
    public static void main(String[] args) {
        System.out.println("Hello from jfu!");
    }
}
```

### Commands
```bash
# Build
jfu build HelloWorld.java

# Run
jfu run HelloWorld.java
```

**Output:**
```
🔄 Checking dependencies...
⚡ Compiling 1 file(s)...
  ⚡ HelloWorld.java
✅ Build complete (1 compiled)

🚀 Running HelloWorld...

Hello from jfu!
```

---

## Project with Dependencies

A project where one class depends on another:

### Calculator.java
```java
/*
dependent "MathUtils.java"
*/
public class Calculator {
    public static void main(String[] args) {
        MathUtils utils = new MathUtils();
        
        int sum = utils.add(5, 3);
        int product = utils.multiply(4, 7);
        
        System.out.println("Sum: " + sum);
        System.out.println("Product: " + product);
    }
}
```

### MathUtils.java
```java
public class MathUtils {
    public int add(int a, int b) {
        return a + b;
    }
    
    public int multiply(int a, int b) {
        return a * b;
    }
}
```

### Commands
```bash
# View dependency tree
jfu tree Calculator.java

# Build and run
jfu run Calculator.java
```

**Tree Output:**
```
📊 Dependency Tree:

📦 Calculator.java
  └─ MathUtils.java
```

---

## Multiple Classes

A more complex example with multiple dependencies:

### App.java
```java
/*
dependent "Database.java"
dependent "Logger.java"
dependent "Config.java"
*/
public class App {
    public static void main(String[] args) {
        Config config = new Config();
        Logger logger = new Logger(config);
        Database db = new Database(config, logger);
        
        logger.info("Application starting...");
        db.connect();
        db.query("SELECT * FROM users");
        db.disconnect();
        logger.info("Application finished.");
    }
}
```

### Database.java
```java
/*
dependent "Logger.java"
dependent "Config.java"
*/
public class Database {
    private Config config;
    private Logger logger;
    
    public Database(Config config, Logger logger) {
        this.config = config;
        this.logger = logger;
    }
    
    public void connect() {
        logger.info("Connecting to " + config.getDatabaseUrl());
    }
    
    public void query(String sql) {
        logger.info("Executing: " + sql);
    }
    
    public void disconnect() {
        logger.info("Disconnecting from database");
    }
}
```

### Logger.java
```java
/*
dependent "Config.java"
*/
public class Logger {
    private Config config;
    
    public Logger(Config config) {
        this.config = config;
    }
    
    public void info(String message) {
        if (config.isVerbose()) {
            System.out.println("[INFO] " + message);
        }
    }
}
```

### Config.java
```java
public class Config {
    public String getDatabaseUrl() {
        return "jdbc:mysql://localhost:3306/mydb";
    }
    
    public boolean isVerbose() {
        return true;
    }
}
```

### Build Output
```bash
jfu build App.java --verbose
```

```
⚙️ Loaded configuration from jfu.toml
🔄 Checking dependencies...
📊 Dependency graph:
  Config.java -> []
  Logger.java -> ["Config.java"]
  Database.java -> ["Logger.java", "Config.java"]
  App.java -> ["Database.java", "Logger.java", "Config.java"]
📋 Build order: ["Config.java", "Logger.java", "Database.java", "App.java"]
⚡ Compiling 4 file(s)...
  🔨 Compiling Config.java...
  🔨 Compiling Logger.java...
  🔨 Compiling Database.java...
  🔨 Compiling App.java...
✅ Build complete (4 compiled)
```

---

## Incremental Build Demo

Demonstrating jfu's intelligent caching:

### Initial Build
```bash
jfu build App.java
```

```
🔄 Checking dependencies...
⚡ Compiling 4 file(s)...
  ⚡ Config.java
  ⚡ Logger.java
  ⚡ Database.java
  ⚡ App.java
✅ Build complete (4 compiled)
```

### Rebuild Without Changes
```bash
jfu build App.java
```

```
🔄 Checking dependencies...
✅ Everything up to date (skipped 4 files)
```

### Modify One File
Edit `Config.java` to add a method:

```java
public class Config {
    public String getDatabaseUrl() {
        return "jdbc:mysql://localhost:3306/mydb";
    }
    
    public boolean isVerbose() {
        return true;
    }
    
    // New method
    public int getTimeout() {
        return 30;
    }
}
```

### Rebuild After Change
```bash
jfu build App.java
```

```
🔄 Checking dependencies...
⚡ Compiling 1 file(s)...
  ⚡ Config.java
✅ Build complete (1 compiled, 3 skipped)
```

**Note:** Only `Config.java` was recompiled! The other 3 files were skipped.

### Force Rebuild
```bash
jfu build App.java --force
```

```
🔄 Checking dependencies...
⚡ Compiling 4 file(s)...
  ⚡ Config.java
  ⚡ Logger.java
  ⚡ Database.java
  ⚡ App.java
✅ Build complete (4 compiled)
```

---

## Using Configuration File

### jfu.toml
```toml
# Custom directory structure
src_dir = "./src"
out_dir = "./build/classes"
cache_file = "./build/jfu-cache.json"

# JVM options for running
jvm_opts = [
    "-Xmx1024m",
    "-Xms512m",
    "-ea",  # Enable assertions
]
```

### Project Structure
```
project/
├── jfu.toml
├── src/
│   ├── Main.java
│   └── Utils.java
└── build/
    ├── classes/
    │   ├── Main.class
    │   └── Utils.class
    └── jfu-cache.json
```

### Commands
```bash
# jfu automatically uses jfu.toml if present
jfu build Main.java

# Configuration is loaded automatically
jfu run Main.java
```

**Output:**
```
⚙️ Loaded configuration from jfu.toml
🔄 Checking dependencies...
⚡ Compiling 2 file(s)...
  ⚡ Utils.java
  ⚡ Main.java
✅ Build complete (2 compiled)

🚀 Running Main...
(program output here)
```

---

## Advanced Patterns

### Pattern 1: Service Layer Architecture

```
project/
├── test/
│   ├── Main.java          → depends on UserService
│   ├── UserService.java   → depends on UserRepository, Logger
│   ├── UserRepository.java → depends on Database, Logger
│   ├── Database.java      → depends on Config
│   ├── Logger.java        → depends on Config
│   └── Config.java        → no dependencies
```

**Dependency comments:**

```java
// Main.java
/*
dependent "UserService.java"
*/

// UserService.java
/*
dependent "UserRepository.java"
dependent "Logger.java"
*/

// UserRepository.java
/*
dependent "Database.java"
dependent "Logger.java"
*/

// Database.java
/*
dependent "Config.java"
*/

// Logger.java
/*
dependent "Config.java"
*/
```

### Pattern 2: Utility Classes

For utility classes used by many files, declare them as dependencies:

```java
// Main.java
/*
dependent "StringUtils.java"
dependent "FileUtils.java"
dependent "DateUtils.java"
*/
public class Main {
    public static void main(String[] args) {
        String formatted = StringUtils.capitalize("hello");
        String content = FileUtils.readFile("data.txt");
        String date = DateUtils.format(new Date());
    }
}
```

### Pattern 3: Testing and Development

```bash
# During development - use verbose mode
jfu run Main.java --verbose

# Before committing - clean build
jfu clean
jfu build Main.java

# Quick iteration loop
jfu run Main.java  # Fast! Uses cache
```

---

## 🔍 Troubleshooting Examples

### Missing Dependency

**Problem:**
```
⚠️ Dependency not found: Helper.java
```

**Solution:**
Ensure the file exists and is in the correct directory:
```bash
ls test/Helper.java  # Should exist
```

### Circular Dependency

**Problem:**
```
❌ Circular dependency detected involving: ClassB.java
```

**ClassA.java:**
```java
/*
dependent "ClassB.java"
*/
```

**ClassB.java:**
```java
/*
dependent "ClassA.java"  // ❌ Circular!
*/
```

**Solution:**
Refactor to remove the circular dependency. Extract shared code into a third class.

### Compilation Error

**Problem:**
```
❌ Compilation failed:
./test/Main.java:10: error: cannot find symbol
```

**Solution:**
1. Check that all dependencies are declared
2. Verify class names match file names
3. Use `jfu build --verbose` for more details

---

## 🎯 Best Practices

### 1. Dependency Declaration
Always declare dependencies at the top of the file:
```java
/*
dependent "Helper.java"
dependent "Utils.java"
*/
public class Main {
    // ... code
}
```

### 2. Clean Builds
Before committing to version control:
```bash
jfu clean
jfu build Main.java
```

### 3. Use Configuration
For team projects, commit `jfu.toml`:
```toml
src_dir = "./src"
out_dir = "./target/classes"
```

### 4. Leverage Caching
Let jfu's cache speed up your workflow:
```bash
# First build - compiles everything
jfu build Main.java

# Subsequent builds - only recompiles changes
jfu build Main.java  # Super fast!
```

---

## 📊 Performance Comparison

### Without jfu (manual javac)
```bash
# Must manually track dependencies and order
javac -d out Config.java
javac -d out -cp out Logger.java
javac -d out -cp out Database.java
javac -d out -cp out Main.java
java -cp out Main

# Every time, even if nothing changed! 😫
```

### With jfu
```bash
# Once
jfu run Main.java  # Compiles all 4 files

# Subsequent runs
jfu run Main.java  # Skips unchanged files ⚡

# After editing Config.java
jfu run Main.java  # Only recompiles Config.java! 🚀
```

---

## 🎓 Real-World Example: Student Project

A complete university assignment example:

### Project: Simple Banking System

```
banking-system/
├── jfu.toml
└── src/
    ├── BankApp.java
    ├── Account.java
    ├── Transaction.java
    └── Utils.java
```

### BankApp.java
```java
/*
dependent "Account.java"
dependent "Transaction.java"
dependent "Utils.java"
*/
public class BankApp {
    public static void main(String[] args) {
        Account account = new Account("12345", 1000.0);
        
        Transaction deposit = new Transaction("DEPOSIT", 500.0);
        account.process(deposit);
        
        Transaction withdraw = new Transaction("WITHDRAW", 200.0);
        account.process(withdraw);
        
        System.out.println("Final balance: " + 
            Utils.formatCurrency(account.getBalance()));
    }
}
```

### Workflow
```bash
# Initial development
jfu run BankApp.java

# Make changes to Account.java
# (edit file)
jfu run BankApp.java  # Only recompiles Account.java

# Before submission
jfu clean
jfu build BankApp.java
jfu run BankApp.java
```

---

**Happy building with jfu! 🚀**

For more information, see the main [README.md](README.md)