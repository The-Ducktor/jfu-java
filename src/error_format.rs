use colored::*;
use terminal_size::{Width, terminal_size};

use crate::search::get_method_suggestions_with_signatures;
use crate::syntax::highlight_java_code;

/// Get the current terminal width, defaulting to 80 if unable to detect
fn get_terminal_width() -> usize {
    if let Some((Width(w), _)) = terminal_size() {
        w as usize
    } else {
        80 // Default fallback width
    }
}

/// Create a separator line that fits the terminal width
fn separator(width: usize) -> String {
    "─".repeat(width.min(120)) // Cap at 120 for very wide terminals
}

pub fn format_java_errors(error_text: &str) -> String {
    let term_width = get_terminal_width();
    let sep_width = (term_width - 2).max(40); // Leave some margin

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
                    separator(sep_width - 12).yellow() // Subtract space for "Error #N "
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
                    let code_line = lines[i + 1];
                    let trimmed = code_line.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with("^") {
                        formatted.push_str("\n");
                        // Preserve leading whitespace for alignment
                        let leading_spaces = code_line.len() - code_line.trim_start().len();
                        let highlighted_code = highlight_java_code(trimmed);
                        formatted.push_str(&format!("  {}\n", highlighted_code));

                        // Show the caret indicator (line after code) with proper alignment
                        if i + 2 < lines.len() {
                            let caret_line = lines[i + 2];
                            let caret_trimmed = caret_line.trim_start();
                            if caret_trimmed.starts_with("^") {
                                // Calculate the offset: original leading spaces minus what we removed
                                let caret_spaces = caret_line.len() - caret_line.trim_start().len();
                                let offset = if caret_spaces > leading_spaces {
                                    caret_spaces - leading_spaces
                                } else {
                                    0
                                };
                                let aligned_caret =
                                    format!("{}{}", " ".repeat(offset), caret_trimmed);
                                formatted.push_str(&format!("  {}\n", aligned_caret.red().bold()));
                            }
                        }
                    }
                }

                // Show additional context lines (symbol, location info) and collect suggestion data
                let mut method_name_opt = None;
                let mut class_name_opt = None;
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

                        // Extract unknown method name from "symbol: method MethodName(...)"
                        if context_line.starts_with("symbol:") && context_line.contains("method ") {
                            if let Some(method_start) = context_line.find("method ") {
                                let method_part = &context_line[method_start + 7..].trim();
                                // Extract method name (before parenthesis)
                                let method_name =
                                    method_part.split('(').next().unwrap_or("").trim();
                                if !method_name.is_empty() {
                                    method_name_opt = Some(method_name.to_string());
                                }
                            }
                        }

                        // Extract class name from "location: ... type ClassName" or "location: class ClassName"
                        if context_line.starts_with("location:") {
                            // Try to find "type ClassName" first
                            if let Some(type_start) = context_line.find("type ") {
                                let class_part = &context_line[type_start + 5..];
                                let class_name =
                                    class_part.trim().split_whitespace().next().unwrap_or("");
                                if !class_name.is_empty() {
                                    class_name_opt = Some(class_name.to_string());
                                }
                            }
                            // Fall back to "class ClassName"
                            else if let Some(class_start) = context_line.find("class ") {
                                let class_part = &context_line[class_start + 6..];
                                let class_name =
                                    class_part.trim().split_whitespace().next().unwrap_or("");
                                if !class_name.is_empty() {
                                    class_name_opt = Some(class_name.to_string());
                                }
                            }
                        }
                    } else if !context_line.contains(".java:")
                        && !context_line.ends_with(" error")
                        && !context_line.ends_with(" errors")
                    {
                        // Skip javac's error count summary lines like "2 errors"
                        formatted.push_str(&format!("    {}\n", context_line.bright_black()));
                    } else {
                        break;
                    }
                    j += 1;
                }

                // Show method suggestions immediately after this error
                if let (Some(class_name), Some(method_name)) = (class_name_opt, method_name_opt) {
                    let suggestions =
                        get_method_suggestions_with_signatures(&class_name, &method_name);

                    if !suggestions.is_empty() {
                        formatted.push_str("\n");
                        formatted.push_str(&format!(
                            "  {} {}\n",
                            "💡".yellow(),
                            "Did you mean:".yellow().bold()
                        ));
                        formatted.push_str(&format!(
                            "    {} Instead of {}.{}(), try:\n",
                            "→".cyan(),
                            class_name.green(),
                            method_name.red()
                        ));

                        for (_method_name, signature) in suggestions.iter().take(3) {
                            // Syntax highlight the suggestion
                            let highlighted = highlight_java_code(signature);
                            formatted.push_str(&format!("      {} {}\n", "•".cyan(), highlighted));
                        }

                        if suggestions.len() > 3 {
                            formatted.push_str(&format!(
                                "      {} ... and {} more overload(s)\n",
                                "•".bright_black(),
                                suggestions.len() - 3
                            ));
                        }
                    }
                }
            }
        }

        i += 1;
    }

    // Show error count summary
    if error_count > 0 {
        formatted.push_str(&format!("\n{}\n", separator(sep_width).yellow()));
        let error_word = if error_count == 1 { "error" } else { "errors" };
        formatted.push_str(&format!(
            "{} {} {}\n\n",
            "📊".yellow(),
            error_count.to_string().red().bold(),
            error_word.red().bold()
        ));

        formatted.push_str(&format!(
            "{} {}\n",
            "💡".cyan(),
            "Fix the errors above and try again.".cyan()
        ));
    } else {
        // Fallback if we couldn't parse the error format
        formatted.push_str("\n");
        for line in error_text.lines() {
            formatted.push_str(&format!("  {}\n", line.red()));
        }
    }

    formatted
}

pub fn format_runtime_errors(error_text: &str) -> String {
    let term_width = get_terminal_width();
    let sep_width = (term_width - 2).max(40); // Leave some margin

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
        formatted.push_str(&format!("{}\n", separator(sep_width).red()));

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

        formatted.push_str(&format!("\n{}\n", separator(sep_width).red()));
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
            "\n{} {}\n",
            "💥".red(),
            "Runtime Error".red().bold()
        ));
        formatted.push_str(&format!("{}\n", separator(sep_width).red()));

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

        formatted.push_str(&format!("\n{}\n", separator(sep_width).red()));
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
