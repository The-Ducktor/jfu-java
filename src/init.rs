use colored::*;
use std::{fs, path::PathBuf};

pub fn init_config(force: bool) -> Result<(), String> {
    let config_path = PathBuf::from("jfu.toml");

    if config_path.exists() && !force {
        return Err(format!(
            "jfu.toml already exists. Use --force to overwrite."
        ));
    }

    let template = r#"# jfu Configuration File

# Source directory containing your Java files
# Defaults to "." (current directory)
src_dir = "."

# Output directory for compiled .class files
out_dir = "./out"

# Location of the build cache file
cache_file = "./jfu-cache.json"

# Default entrypoint when no file is specified
# This is useful when you have multiple classes with main() methods
entrypoint = "Main.java"

# JVM options to pass when running your program
jvm_opts = ["-Xmx256m"]

# Future features (not yet implemented):
#
# [dependencies]
# # External JAR files to include in classpath
# libs = [
#     "lib/commons-lang3-3.12.0.jar",
# ]
#
# [compiler]
# # Additional javac options
# javac_opts = ["-Xlint:unchecked", "-g"]
"#;

    fs::write(&config_path, template).map_err(|e| format!("Failed to create jfu.toml: {}", e))?;

    println!("{} Created jfu.toml", "✅".green());
    println!("\n{}", "Configuration file created with defaults:".cyan());
    println!("  {} src_dir = \".\"", "•".blue());
    println!("  {} out_dir = \"./out\"", "•".blue());
    println!("  {} cache_file = \"./jfu-cache.json\"", "•".blue());
    println!("  {} entrypoint = \"Main.java\"", "•".blue());
    println!("  {} jvm_opts = [\"-Xmx256m\"]", "•".blue());
    println!(
        "\n{}",
        "Edit jfu.toml to customize your project settings.".cyan()
    );

    Ok(())
}
