//! Search functionality for Java API documentation using embedded docs

use crate::{
    docs::{get_docs, Class, Method, Package},
    fuzzy::smart_match_methods,
    syntax::highlight_java_code,
};
use colored::*;
use std::io::{self, Write};

/// Search for a class by name and display its information
pub fn search_class(class_name: &str, verbose: bool) -> Result<(), String> {
    let docs = get_docs();

    // Try to find the class
    match docs.get_class_with_package(class_name) {
        Some((package, class)) => {
            display_class_info(package, class, verbose);
            Ok(())
        }
        None => {
            // If not found, try searching for partial matches
            let results = docs.search_classes(class_name);
            if results.is_empty() {
                Err(format!("Class '{}' not found in Java docs", class_name))
            } else {
                println!(
                    "{} No exact match found. Did you mean one of these?\n",
                    "ℹ️".blue()
                );
                for (fqn, class) in results.iter().take(10) {
                    println!("  {} {}", "•".cyan(), fqn.green());
                    if verbose {
                        println!("    Methods: {}", class.methods.len());
                    }
                }
                if results.len() > 10 {
                    println!("\n  ... and {} more", results.len() - 10);
                }
                Ok(())
            }
        }
    }
}

/// Search for methods in a specific class
pub fn search_methods(class_name: &str, method_query: Option<&str>) -> Result<(), String> {
    let docs = get_docs();

    match docs.get_class_with_package(class_name) {
        Some((package, class)) => {
            println!(
                "\n{} Methods in {}.{}:\n",
                "📚".cyan(),
                package.package.yellow(),
                class.name.green().bold()
            );

            let methods: Vec<&Method> = if let Some(query) = method_query {
                // Use the consolidated fuzzy matching logic
                smart_match_methods(query, &class.methods)
            } else {
                class.methods.iter().collect()
            };

            if methods.is_empty() {
                println!("  {} No methods found", "ℹ️".blue());
                return Ok(());
            }

            for method in &methods {
                display_method(method, true);
            }

            println!(
                "\n{} Total: {} method(s)",
                "✓".green(),
                methods.len().to_string().bold()
            );

            Ok(())
        }
        None => Err(format!("Class '{}' not found in Java docs", class_name)),
    }
}

/// Interactive search mode
pub fn interactive_search() -> Result<(), String> {
    let docs = get_docs();

    println!("{}", "╔═══════════════════════════════════════════╗".cyan());
    println!("{}", "║   Java API Documentation Search          ║".cyan());
    println!("{}", "╚═══════════════════════════════════════════╝".cyan());
    println!();
    println!("Commands:");
    println!(
        "  {} <class>           - Search for a class",
        "class".green()
    );
    println!(
        "  {} <class>.<method>  - Search for a specific method",
        "method".green()
    );
    println!(
        "  {} <query>           - Search all classes",
        "search".green()
    );
    println!(
        "  {} <package>         - List all classes in package",
        "pkg".green()
    );
    println!("  {}                     - Exit", "quit".green());
    println!();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("{} ", "jfu>".blue().bold());
        stdout.flush().unwrap();

        let mut input = String::new();
        stdin.read_line(&mut input).map_err(|e| e.to_string())?;

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "quit" | "exit" | "q" => {
                println!("Goodbye! 👋");
                break;
            }
            "class" => {
                if parts.len() < 2 {
                    println!("{} Usage: class <name>", "⚠️".yellow());
                    continue;
                }
                let _ = search_class(parts[1], true);
            }
            "method" => {
                if parts.len() < 2 {
                    println!("{} Usage: method <class>.<method>", "⚠️".yellow());
                    continue;
                }
                if let Some((class_name, method_name)) = parts[1].split_once('.') {
                    if let Some(method) = docs.get_class_method(class_name, method_name) {
                        println!(
                            "\n{} Method: {}.{}\n",
                            "📚".cyan(),
                            class_name.green(),
                            method_name.yellow()
                        );
                        display_method(method, true);
                    } else {
                        println!("{} Method not found", "❌".red());
                    }
                } else {
                    println!(
                        "{} Usage: method <class>.<method> (e.g., String.substring)",
                        "⚠️".yellow()
                    );
                }
            }
            "search" => {
                if parts.len() < 2 {
                    println!("{} Usage: search <query>", "⚠️".yellow());
                    continue;
                }
                let query = parts[1..].join(" ");
                let results = docs.search_classes(&query);
                if results.is_empty() {
                    println!("{} No results found for '{}'", "ℹ️".blue(), query);
                } else {
                    println!("\n{} Found {} result(s):\n", "🔍".cyan(), results.len());
                    for (fqn, class) in results.iter().take(20) {
                        println!("  {} {}", "•".cyan(), fqn.green());
                        println!("    {} method(s)", class.methods.len());
                    }
                    if results.len() > 20 {
                        println!("\n  ... and {} more", results.len() - 20);
                    }
                }
            }
            "pkg" => {
                if parts.len() < 2 {
                    println!("{} Usage: pkg <package>", "⚠️".yellow());
                    continue;
                }
                let pkg_name = parts[1];
                if let Some(package) = docs.get_package(pkg_name) {
                    display_package_info(package);
                } else {
                    println!("{} Package '{}' not found", "❌".red(), pkg_name);
                }
            }
            "help" | "?" => {
                println!("\nAvailable commands:");
                println!(
                    "  {} <class>           - Search for a class",
                    "class".green()
                );
                println!(
                    "  {} <class>.<method>  - Search for a specific method",
                    "method".green()
                );
                println!(
                    "  {} <query>           - Search all classes",
                    "search".green()
                );
                println!(
                    "  {} <package>         - List all classes in package",
                    "pkg".green()
                );
                println!("  {}                     - Exit\n", "quit".green());
            }
            _ => {
                // Default to class search
                let _ = search_class(input, false);
            }
        }
        println!();
    }

    Ok(())
}

/// Display detailed information about a class
fn display_class_info(package: &Package, class: &Class, verbose: bool) {
    println!();
    println!("{}", "╔═══════════════════════════════════════════╗".cyan());
    println!(
        "║ {} Class: {}.{}",
        "📦".cyan(),
        package.package.yellow(),
        class.name.green().bold()
    );
    println!("{}", "╚═══════════════════════════════════════════╝".cyan());
    println!();

    if !package.description.is_empty() {
        println!("{} {}", "Package:".bold(), package.description);
        println!();
    }

    println!("{} {} method(s)", "Methods:".bold(), class.methods.len());
    println!();

    for method in &class.methods {
        display_method(method, verbose);
    }
}

/// Display information about a method
fn display_method(method: &Method, show_descriptions: bool) {
    let overload_count = method.overloads.len();
    let overload_text = if overload_count > 1 {
        format!(" ({} overloads)", overload_count)
    } else {
        String::new()
    };

    println!(
        "  {} {}{}",
        "▸".cyan(),
        method.name.yellow().bold(),
        overload_text.dimmed()
    );

    for (idx, overload) in method.overloads.iter().enumerate() {
        let is_last = idx == method.overloads.len() - 1;
        let prefix = if overload_count > 1 {
            if is_last {
                "  └─"
            } else {
                "  ├─"
            }
        } else {
            "    "
        };

        let deprecated_marker = if overload.deprecated {
            format!(" {}", "[DEPRECATED]".red())
        } else {
            String::new()
        };

        println!(
            "{}{}{}",
            prefix.cyan(),
            highlight_java_code(&overload.signature),
            deprecated_marker
        );

        if show_descriptions && !overload.description.is_empty() {
            // Format description with indentation
            let desc_lines: Vec<&str> = overload.description.lines().collect();
            let continuation = if overload_count > 1 {
                if is_last {
                    "     "
                } else {
                    "  │  "
                }
            } else {
                "     "
            };

            // Only show first 3 lines of description for compactness
            for (line_idx, line) in desc_lines.iter().take(3).enumerate() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    if line_idx == 0 {
                        println!("{}{}", continuation.cyan(), trimmed.dimmed());
                    } else {
                        println!("{}{}", continuation.cyan(), trimmed.dimmed());
                    }
                }
            }

            // If there are more lines, indicate truncation
            if desc_lines.len() > 3 {
                println!("{}...", continuation.cyan());
            }
        }
    }
    println!();
}

/// Display package information
fn display_package_info(package: &Package) {
    println!();
    println!("{}", "╔═══════════════════════════════════════════╗".cyan());
    println!(
        "║ {} Package: {}",
        "📦".cyan(),
        package.package.yellow().bold()
    );
    println!("{}", "╚═══════════════════════════════════════════╝".cyan());
    println!();

    if !package.description.is_empty() {
        println!("{}", package.description);
        println!();
    }

    println!("{} {} classes:", "Classes:".bold(), package.classes.len());
    println!();

    for class in &package.classes {
        println!(
            "  {} {} ({} methods)",
            "•".cyan(),
            class.name.green(),
            class.methods.len().to_string().dimmed()
        );
    }
    println!();
}

/// Get suggestions for a class name (used by error formatting)
pub fn get_class_suggestions(class_name: &str) -> Vec<String> {
    let docs = get_docs();

    // Search for similar class names
    let results = docs.search_classes(class_name);

    // Return up to 5 suggestions
    results.iter().take(5).map(|(fqn, _)| fqn.clone()).collect()
}

/// Get suggestions for a method name in a specific class (case-insensitive)
/// Returns list of suggested method names
pub fn get_method_suggestions(class_name: &str, method_name: &str) -> Vec<String> {
    let docs = get_docs();

    // Try to find the class (case-insensitive)
    let class = docs.get_class(class_name);

    if class.is_none() {
        return Vec::new();
    }

    let class = class.unwrap();

    // Use the consolidated fuzzy matching logic
    crate::fuzzy::get_method_suggestions(method_name, &class.methods)
}

/// Get detailed method suggestions with signature info
/// Returns list of tuples (method_name, signature) including all overloads
pub fn get_method_suggestions_with_signatures(
    class_name: &str,
    method_name: &str,
) -> Vec<(String, String)> {
    let docs = get_docs();

    let class = docs.get_class(class_name);

    if class.is_none() {
        return Vec::new();
    }

    let class = class.unwrap();

    // Use the consolidated fuzzy matching logic
    crate::fuzzy::get_method_suggestions_with_signatures(method_name, &class.methods, 3)
}
