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
    let mut unknown_classes = Vec::new();
    let mut method_suggestions_map: Vec<(String, String, Vec<(String, String)>)> = Vec::new();

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
                        // Preserve leading whitespace for alignment
                        let leading_spaces = code_line.len() - code_line.trim_start().len();
                        let highlighted_code = highlight_java_code(trimmed);
                        formatted.push_str(&format!("\n  {}\n", highlighted_code));

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

                        // Extract unknown class name from "symbol: class ClassName"
                        if context_line.starts_with("symbol:") && context_line.contains("class ") {
                            if let Some(class_start) = context_line.find("class ") {
                                let class_name = &context_line[class_start + 6..]
                                    .trim()
                                    .split_whitespace()
                                    .next()
                                    .unwrap_or("");
                                if !class_name.is_empty()
                                    && class_name.chars().next().unwrap_or('a').is_uppercase()
                                {
                                    unknown_classes.push(class_name.to_string());
                                }
                            }
                        }

                        // Extract unknown method name from "symbol: method MethodName(...)"
                        if context_line.starts_with("symbol:") && context_line.contains("method ") {
                            if let Some(method_start) = context_line.find("method ") {
                                let method_part = &context_line[method_start + 7..].trim();
                                // Extract method name (before parenthesis)
                                let method_name =
                                    method_part.split('(').next().unwrap_or("").trim();

                                if !method_name.is_empty() {
                                    // Look for "location: ... type ClassName" or "location: class ClassName" in subsequent lines
                                    let mut k = j + 1;
                                    while k < lines.len() && k < j + 5 {
                                        let loc_line = lines[k].trim();
                                        if loc_line.starts_with("location:") {
                                            let mut class_name_opt = None;

                                            // Try to find "type ClassName" first
                                            if let Some(type_start) = loc_line.find("type ") {
                                                class_name_opt = Some(&loc_line[type_start + 5..]);
                                            }
                                            // Fall back to "class ClassName"
                                            else if let Some(class_start) =
                                                loc_line.find("class ")
                                            {
                                                class_name_opt = Some(&loc_line[class_start + 6..]);
                                            }

                                            if let Some(class_part) = class_name_opt {
                                                let class_name = class_part
                                                    .trim()
                                                    .split_whitespace()
                                                    .next()
                                                    .unwrap_or("");

                                                if !class_name.is_empty() {
                                                    // Get method suggestions
                                                    let suggestions =
                                                        get_method_suggestions_with_signatures(
                                                            class_name,
                                                            method_name,
                                                        );

                                                    if !suggestions.is_empty() {
                                                        method_suggestions_map.push((
                                                            class_name.to_string(),
                                                            method_name.to_string(),
                                                            suggestions,
                                                        ));
                                                    }
                                                }
                                            }
                                            break;
                                        }
                                        k += 1;
                                    }
                                }
                            }
                        }
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
            formatted.push_str(&format!("\n{}\n", separator(sep_width).yellow()));
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
        // Show method suggestions if any were found
        if !method_suggestions_map.is_empty() {
            formatted.push_str(&format!("\n{}\n", separator(sep_width).cyan()));
            formatted.push_str(&format!(
                "{} {}\n\n",
                "💡".yellow(),
                "Did you mean:".yellow().bold()
            ));

            for (class_name, wrong_method, suggestions) in method_suggestions_map {
                formatted.push_str(&format!(
                    "  {} Instead of {}.{}(), try:\n",
                    "→".cyan(),
                    class_name.green(),
                    wrong_method.red()
                ));

                for (_method_name, signature) in suggestions.iter().take(5) {
                    formatted.push_str(&format!("    {} {}\n", "•".cyan(), signature.green()));
                }

                if suggestions.len() > 5 {
                    formatted.push_str(&format!(
                        "    {} ... and {} more overload(s)\n",
                        "•".bright_black(),
                        suggestions.len() - 5
                    ));
                }
                formatted.push_str("\n");
            }
        }

        formatted.push_str(&format!(
            "{} Fix the errors above and try again.\n",
            "💡".cyan()
        ));
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
            "{} {}\n",
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
