use colored::*;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_src_dir")]
    pub src_dir: PathBuf,
    #[serde(default = "default_out_dir")]
    pub out_dir: PathBuf,
    #[serde(default = "default_cache_file")]
    pub cache_file: PathBuf,
    #[serde(default)]
    pub jvm_opts: Vec<String>,
    #[serde(default)]
    pub entrypoint: Option<String>,
    #[serde(default)]
    pub auto_include_implicit_deps: bool,
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
        }
    }
}

impl Config {
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
