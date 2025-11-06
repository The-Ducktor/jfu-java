use colored::*;

use crate::syntax::highlight_java_code;

pub fn format_java_errors(error_text: &str) -> String {
    let mut formatted = String::new();
    formatted.push_str(&format!(
        "\n{} {}\n",
        "💥".red(),
        "Compilation Failed".red().bold()
    ));

    let lines: Vec<&str> = error_text.lines().collect();
    let mut i = 0;
    let mut error_count = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Check if this is an error line (typically starts with file path)
        if line.contains(".java:") && line.contains(": error:") {
            error_count += 1;

            // Parse the error line: ./test/File.java:10: error: message
            if let Some(colon_pos) = line.find(": error:") {
                let file_and_line = &line[..colon_pos];
                let error_msg = &line[colon_pos + 8..].trim();

                formatted.push_str(&format!(
                    "\n{} {}\n",
                    format!("Error #{}", error_count).yellow().bold(),
                    "─".repeat(60).yellow()
                ));

                // Extract file and line number
                if let Some(last_colon) = file_and_line.rfind(':') {
                    let location = &file_and_line[last_colon + 1..];
                    let file_path = &file_and_line[..last_colon];

                    formatted.push_str(&format!("  {} {}\n", "📄".cyan(), file_path.cyan()));
                    formatted.push_str(&format!(
                        "  {} Line {}\n",
                        "📍".yellow(),
                        location.yellow().bold()
                    ));
                    formatted.push_str(&format!("  {} {}\n", "💬".red(), error_msg.white()));
                }

                // Show the problematic code line (next line usually)
                if i + 1 < lines.len() {
                    let code_line = lines[i + 1].trim();
                    if !code_line.is_empty() && !code_line.starts_with("^") {
                        let highlighted_code = highlight_java_code(code_line);
                        formatted.push_str(&format!("\n  {}\n", highlighted_code));
                    }
                }

                // Show the caret indicator (line after code)
                if i + 2 < lines.len() {
                    let caret_line = lines[i + 2].trim();
                    if caret_line.starts_with("^") {
                        formatted.push_str(&format!("  {}\n", caret_line.red().bold()));
                    }
                }

                // Show additional context lines (symbol, location info)
                let mut j = i + 3;
                while j < lines.len() && j < i + 10 {
                    let context_line = lines[j].trim();
                    if context_line.is_empty() {
                        break;
                    }
                    if context_line.starts_with("symbol:") || context_line.starts_with("location:")
                    {
                        formatted.push_str(&format!(
                            "    {} {}\n",
                            "•".blue(),
                            context_line.bright_black()
                        ));
                    } else if !context_line.contains(".java:") {
                        formatted.push_str(&format!("    {}\n", context_line.bright_black()));
                    } else {
                        break;
                    }
                    j += 1;
                }
            }
        } else if line.contains(" error") && line.ends_with(" error") {
            // Summary line like "1 error" or "3 errors"
            formatted.push_str(&format!("\n{}\n", "─".repeat(70).yellow()));
            formatted.push_str(&format!("{} {}\n", "📊".yellow(), line.red().bold()));
        }

        i += 1;
    }

    if error_count == 0 {
        // Fallback if we couldn't parse the error format
        formatted.push_str("\n");
        for line in error_text.lines() {
            formatted.push_str(&format!("  {}\n", line.red()));
        }
    } else {
        formatted.push_str(&format!(
            "\n{} Fix the errors above and try again.\n",
            "💡".cyan()
        ));
    }

    formatted
}

pub fn format_runtime_errors(error_text: &str) -> String {
    let lines: Vec<&str> = error_text.lines().collect();

    // Check for StackOverflowError (recursion)
    if lines.iter().any(|line| line.contains("StackOverflowError")) {
        let mut formatted = String::new();
        formatted.push_str(&format!(
            "\n{} {}\n",
            "🔄".red(),
            "Stack Overflow Error - Infinite Recursion Detected!"
                .red()
                .bold()
        ));
        formatted.push_str(&format!("{}\n", "─".repeat(70).red()));

        formatted.push_str(&format!(
            "\n  {} {}\n",
            "💡".yellow(),
            "This usually happens when:".yellow().bold()
        ));
        formatted.push_str("    • A method calls itself without a proper base case\n");
        formatted.push_str("    • Methods call each other in a circular pattern\n");
        formatted.push_str("    • A loop condition never becomes false\n\n");

        // Find the repeating pattern in stack trace
        let at_lines: Vec<&str> = lines
            .iter()
            .filter(|line| line.trim().starts_with("at "))
            .take(10) // Show first 10 stack frames
            .copied()
            .collect();

        if !at_lines.is_empty() {
            formatted.push_str(&format!(
                "  {} {}\n\n",
                "📍".cyan(),
                "Top of call stack (most recent calls):".cyan().bold()
            ));

            for (i, line) in at_lines.iter().enumerate() {
                let trimmed = line.trim();
                if trimmed.contains(".java:") {
                    formatted.push_str(&format!("    {}. {}\n", i + 1, trimmed.cyan()));
                } else {
                    formatted.push_str(&format!("    {}. {}\n", i + 1, trimmed.bright_black()));
                }
            }

            // Count total lines to show recursion depth
            let total_at_lines = lines
                .iter()
                .filter(|line| line.trim().starts_with("at "))
                .count();

            if total_at_lines > 10 {
                formatted.push_str(&format!(
                    "\n    {} ... and {} more recursive calls\n",
                    "↓".yellow(),
                    total_at_lines - 10
                ));
            }
        }

        formatted.push_str(&format!("\n{}\n", "─".repeat(70).red()));
        formatted.push_str(&format!(
            "{} {} to prevent infinite recursion.\n",
            "🔧".green(),
            "Add a base case or exit condition".green().bold()
        ));

        return formatted;
    }

    // Check if it's a Java exception
    if lines.iter().any(|line| line.contains("Exception")) {
        let mut formatted = String::new();
        formatted.push_str(&format!(
            "{} {}\n",
            "💥".red(),
            "Runtime Error".red().bold()
        ));
        formatted.push_str(&format!("{}\n", "─".repeat(70).red()));

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Exception type line
            if trimmed.contains("Exception") && i == 0 {
                formatted.push_str(&format!("\n  {} {}\n", "🔥".yellow(), trimmed.red().bold()));
            }
            // Stack trace lines
            else if trimmed.starts_with("at ") {
                // Highlight our code vs library code
                if trimmed.contains(".java:") {
                    formatted.push_str(&format!("    {} {}\n", "→".cyan(), trimmed.cyan()));
                } else {
                    formatted.push_str(&format!(
                        "    {} {}\n",
                        "·".bright_black(),
                        trimmed.bright_black()
                    ));
                }
            }
            // Caused by
            else if trimmed.starts_with("Caused by:") {
                formatted.push_str(&format!("\n  {} {}\n", "↳".yellow(), trimmed.yellow()));
            }
            // Other lines
            else if !trimmed.is_empty() {
                formatted.push_str(&format!("  {}\n", trimmed.red()));
            }
        }

        formatted.push_str(&format!("\n{}\n", "─".repeat(70).red()));
        formatted.push_str(&format!(
            "{} Check the stack trace above to find the issue.\n",
            "💡".cyan()
        ));

        formatted
    } else {
        // Not a standard exception, return as-is but colored
        format!(
            "{} {}\n{}",
            "⚠️".yellow(),
            "Error:".yellow().bold(),
            error_text.red()
        )
    }
}
