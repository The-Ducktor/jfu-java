use colored::*;
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

use crate::build::{build_files, BuildContext};
use crate::classpath::{build_classpath_string, resolve_classpath};
use crate::error_format::format_runtime_errors;

/// Run a compiled Java class while preserving TTY behaviour for stdout/stderr.
/// This function:
/// - inherits stdin and stdout so the child sees the real terminal (fixes Scanner/TTY issues)
/// - pipes stderr so we can both stream it live and capture it for pretty formatting
/// - resolves and applies the classpath for external JAR files
pub fn run_file(ctx: &BuildContext, main_file: &str) -> Result<(), String> {
    // Build first
    build_files(ctx, main_file)?;

    // Determine class name from file (strip .java and path)
    let base_name = main_file
        .strip_suffix(".java")
        .ok_or_else(|| format!("Invalid Java file: {}", main_file))?;

    // Extract just the filename without directory path
    let class_name = base_name
        .split('/')
        .last()
        .or_else(|| base_name.split('\\').last())
        .unwrap_or(base_name);

    println!("     {} `java {}`", "Running".green().bold(), class_name);

    // Resolve classpath from glob patterns and build the classpath string
    let resolved_jars = resolve_classpath(&ctx.config.classpath);
    let classpath = build_classpath_string(&resolved_jars, &ctx.config.out_dir);

    // Configure command
    let mut cmd = Command::new("java");

    // Add classpath if there are JARs or we need to include the output directory
    if !resolved_jars.is_empty() || !ctx.config.classpath.is_empty() {
        cmd.arg("-cp").arg(&classpath);
    } else {
        // Always include output directory even without external JARs
        cmd.arg("-cp").arg(&ctx.config.out_dir);
    }

    for opt in &ctx.config.jvm_opts {
        cmd.arg(opt);
    }

    cmd.arg(class_name);

    // Important: inherit stdin and stdout to preserve TTY behaviour for interactive programs.
    // Only pipe stderr so we can capture/format it.
    let mut child = cmd
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run java: {}", e))?;

    // If stderr was piped, stream it live and also accumulate for formatted output later.
    let (tx, rx) = mpsc::channel();
    if let Some(mut stderr_handle) = child.stderr.take() {
        let tx = tx.clone();
        thread::spawn(move || {
            let mut buf = [0u8; 8192];
            let mut accumulated: Vec<u8> = Vec::new();
            let mut err_out = io::stderr();

            loop {
                match stderr_handle.read(&mut buf) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        // write raw bytes to parent's stderr immediately
                        let _ = err_out.write_all(&buf[..n]);
                        let _ = err_out.flush();

                        // accumulate for formatted output later
                        accumulated.extend_from_slice(&buf[..n]);
                    }
                    Err(_) => break,
                }
            }

            // Convert accumulated bytes to String (lossy) and send back
            let s = String::from_utf8_lossy(&accumulated).into_owned();
            let _ = tx.send(s);
        });
    }

    // Wait for the child process to exit
    let status = child
        .wait()
        .map_err(|e| format!("Failed to wait for child process: {}", e))?;

    // Receive accumulated stderr (if any)
    let stderr_content = match rx.recv() {
        Ok(s) => s,
        Err(_) => String::new(),
    };

    // If there was any stderr output, show the formatted version as well.
    // This duplicates the raw stderr already printed live, but provides the pretty/structured output.
    if !stderr_content.is_empty() {
        eprintln!("\n{}", format_runtime_errors(&stderr_content));
    }

    if !status.success() {
        return Err(format!(
            "Program exited with status code: {}",
            status.code().unwrap_or(-1)
        ));
    }

    Ok(())
}
