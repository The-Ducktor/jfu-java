use colored::*;
use std::process::Command;

use crate::build::{BuildContext, build_files};
use crate::error_format::format_runtime_errors;

pub fn run_file(ctx: &BuildContext, main_file: &str) -> Result<(), String> {
    // First, build
    build_files(ctx, main_file)?;

    // Extract class name (Main.java -> Main)
    let class_name = main_file
        .strip_suffix(".java")
        .ok_or_else(|| format!("Invalid Java file: {}", main_file))?;

    println!("\n{} Running {}...\n", "🚀".green(), class_name);

    // Run the Java program with optional JVM opts
    let mut cmd = Command::new("java");
    cmd.arg("-cp").arg(&ctx.config.out_dir);

    // Add JVM options if specified
    for opt in &ctx.config.jvm_opts {
        cmd.arg(opt);
    }

    cmd.arg(class_name);

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run java: {}", e))?;

    // Print stdout
    let stdout = String::from_utf8_lossy(&output.stdout);
    print!("{}", stdout);

    // Print stderr if any
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        eprintln!("\n{}", format_runtime_errors(&stderr));
    }

    if !output.status.success() {
        return Err(format!(
            "Program exited with status code: {}",
            output.status.code().unwrap_or(-1)
        ));
    }

    Ok(())
}
