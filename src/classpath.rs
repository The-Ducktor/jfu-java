//! Classpath resolution for external JAR files.
//!
//! This module expands glob patterns in classpath entries to find JAR files.
//! It supports:
//! - Simple globs: `libs/*.jar`
//! - Recursive globs: `vendor/**/lib.jar`
//! - Multiple files matching a pattern

use glob::glob;
use std::path::PathBuf;

/// Resolve classpath entries by expanding glob patterns.
///
/// Takes a list of glob patterns (e.g., `["libs/*.jar", "vendor/**/lib*.jar"]`)
/// and expands them to actual file paths. Handles errors gracefully by skipping
/// invalid patterns.
///
/// # Arguments
/// * `patterns` - List of glob patterns to expand
///
/// # Returns
/// A vector of absolute paths to matching JAR files
pub fn resolve_classpath(patterns: &[String]) -> Vec<PathBuf> {
    let mut jars = Vec::new();

    for pattern in patterns {
        match glob(pattern) {
            Ok(paths) => {
                for entry in paths {
                    if let Ok(path) = entry {
                        // Only include files, not directories
                        if path.is_file() {
                            jars.push(path);
                        }
                    }
                }
            }
            Err(e) => {
                use colored::*;
                eprintln!(
                    "     {} Invalid classpath pattern '{}': {}",
                    "Warning:".yellow().bold(),
                    pattern,
                    e
                );
            }
        }
    }

    jars
}

/// Build a classpath string from resolved JAR files.
///
/// Joins JAR file paths with the platform-specific path separator (`:` on Unix, `;` on Windows).
/// Also includes the provided output directory for compiled classes.
///
/// # Arguments
/// * `jars` - List of JAR file paths
/// * `out_dir` - Output directory for compiled .class files
///
/// # Returns
/// A classpath string suitable for passing to javac/java with `-cp` flag
pub fn build_classpath_string(jars: &[PathBuf], out_dir: &PathBuf) -> String {
    let mut classpath = vec![out_dir.to_string_lossy().to_string()];

    for jar in jars {
        classpath.push(jar.to_string_lossy().to_string());
    }

    // Use platform-specific separator
    let separator = if cfg!(windows) { ";" } else { ":" };
    classpath.join(separator)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_classpath_string_unix() {
        let jars = vec![
            PathBuf::from("libs/lib1.jar"),
            PathBuf::from("libs/lib2.jar"),
        ];
        let out_dir = PathBuf::from("./out");
        let cp = build_classpath_string(&jars, &out_dir);

        // On Unix, separator is ":"
        #[cfg(not(windows))]
        assert!(cp.contains("./out:"));
        assert!(cp.contains("libs/lib1.jar"));
        assert!(cp.contains("libs/lib2.jar"));
    }
}
