# jfu Test Files

This directory contains example Java files for testing and demonstrating jfu's features.

## Test Files

### Basic Examples

- **Main.java** - Demonstrates dependency management with multiple files
- **Test.java** - Simple valid Java program with basic control flow
- **WithExternalLibrary.java** - Shows how to use external libraries with classpath

### Dependency Files (used by Main.java)

- **Runner.java** - Example dependency class
- **Cool.java** - Another example dependency
- **Helper.java** - Helper class with methods
- **HelperTest.java** - Test utility class
- **Utils.java** - Utility functions

### Error Example Files

These files are designed to show jfu's error formatting capabilities:

#### Compilation Errors
- **CompilationErrorExample.java** - Shows typo in method name (toUppercaes instead of toUpperCase)
  - Run with: `jfu build CompilationErrorExample.java`
  - Demonstrates: Colored error messages with suggestions

#### Runtime Errors
- **RuntimeErrorDemo.java** - NullPointerException example
  - Run with: `jfu run RuntimeErrorDemo.java`
  - Demonstrates: Runtime error with code context display

- **ArrayErrorDemo.java** - ArrayIndexOutOfBoundsException example
  - Run with: `jfu run ArrayErrorDemo.java`
  - Demonstrates: Array access error with line highlighting

- **ClassCastErrorDemo.java** - ClassCastException example
  - Run with: `jfu run ClassCastErrorDemo.java`
  - Demonstrates: Type casting error with context

- **RecursionErrorDemo.java** - StackOverflowError example
  - Run with: `jfu run RecursionErrorDemo.java`
  - Demonstrates: Special handling for infinite recursion with helpful hints

## Configuration

- **jfu.toml** - Example configuration file showing:
  - Source and output directories
  - JVM options
  - Classpath for external JARs with glob patterns
  - Automatic implicit dependency detection

## Running Tests

```bash
# Run a simple program
jfu run Test.java

# Run the main demo (with dependencies)
jfu run Main.java

# Build without running
jfu build Test.java

# Show dependency tree
jfu tree Main.java

# Clean build artifacts
jfu clean

# Force rebuild
jfu -f run Test.java

# Verbose output
jfu -v run Test.java
```

## Testing Error Formatting

To see the improved error messages:

1. **Build error example:**
   ```bash
   jfu build CompilationErrorExample.java
   ```
   Shows formatted compilation error with:
   - File location and line number
   - The problematic code
   - Caret pointing to error location
   - Helpful suggestions

2. **Runtime error example:**
   ```bash
   jfu run RuntimeErrorDemo.java
   ```
   Shows formatted runtime error with:
   - Exception type and message
   - Stack trace with user code highlighted
   - **Code context** - The actual line that threw the exception
   - Helpful hints for common errors

3. **Recursion error example:**
   ```bash
   jfu run RecursionErrorDemo.java
   ```
   Shows special handling for StackOverflowError with:
   - Call stack visualization
   - Count of recursive calls
   - Hints about adding base cases

## External JAR Example

To test JAR classpath support:

1. Create a `libs/` directory:
   ```bash
   mkdir libs
   ```

2. Place some JAR files there (or use standard library JARs)

3. Update `jfu.toml`:
   ```toml
   classpath = ["libs/*.jar"]
   ```

4. Run jfu - it will automatically use the JARs:
   ```bash
   jfu run WithExternalLibrary.java
   ```

jfu will expand the glob pattern and pass the classpath to both javac and java.
