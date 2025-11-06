use colored::*;
use std::fs;

use crate::config::Config;

pub fn clean(config: &Config) -> Result<(), String> {
    let mut cleaned = Vec::new();

    if config.out_dir.exists() {
        fs::remove_dir_all(&config.out_dir)
            .map_err(|e| format!("Failed to remove output directory: {}", e))?;
        cleaned.push(config.out_dir.display().to_string());
    }

    if config.cache_file.exists() {
        fs::remove_file(&config.cache_file)
            .map_err(|e| format!("Failed to remove cache file: {}", e))?;
        cleaned.push(config.cache_file.display().to_string());
    }

    if cleaned.is_empty() {
        println!("{} Nothing to clean", "✨".cyan());
    } else {
        println!("{} Cleaned build artifacts:", "🧹".green());
        for item in cleaned {
            println!("  {} {}", "✓".green(), item);
        }
    }

    Ok(())
}
