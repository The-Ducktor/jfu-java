//! Configuration management for jfu projects.
//!
//! This module handles loading and parsing `jfu.toml` files, which configure:
//! - Source and output directories
//! - Cache location
//! - JVM options
//! - Project entrypoint
//! - Implicit dependency behavior
//! - JAR classpath (for external dependencies)

use colored::*;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

/// Project configuration loaded from jfu.toml or defaults.
///
/// All fields have sensible defaults, so even a missing jfu.toml file will result
/// in a valid configuration that builds Java files in the current directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Directory containing Java source files (default: ".")
    #[serde(default = "default_src_dir")]
    pub src_dir: PathBuf,
    /// Directory where compiled .class files are written (default: "./out")
    #[serde(default = "default_out_dir")]
    pub out_dir: PathBuf,
    /// Path to the build cache JSON file (default: "./jfu-cache.json")
    #[serde(default = "default_cache_file")]
    pub cache_file: PathBuf,
    /// JVM options passed to `java` command (e.g., "-Xmx1g")
    #[serde(default)]
    pub jvm_opts: Vec<String>,
    /// Default entrypoint class to run if none is specified (optional)
    #[serde(default)]
    pub entrypoint: Option<String>,
    /// If true, automatically include all public classes in src_dir as dependencies
    #[serde(default)]
    pub auto_include_implicit_deps: bool,
    /// Classpath entries for external JAR files (e.g., ["libs/*.jar", "vendor/lib.jar"])
    /// Supports glob patterns including ** for recursive matching
    #[serde(default)]
    pub classpath: Vec<String>,
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
            auto_include_implicit_deps: false,
            classpath: Vec::new(),
        }
    }
}

impl Config {
    /// Load configuration from `jfu.toml` in the current directory.
    ///
    /// If the file exists and is valid TOML, it's parsed and returned.
    /// If the file doesn't exist or parsing fails, a default configuration is returned
    /// with warnings printed to stderr.
    ///
    /// # Returns
    /// A valid Config object, either loaded from `jfu.toml` or using defaults.
    pub fn load() -> Self {
        let config_path = PathBuf::from("jfu.toml");

        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(config) => {
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
