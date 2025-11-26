//! Example demonstrating the embedded Java docs API
//!
//! This example shows how to use the embedded, compressed Java documentation
//! for quick lookups and searches.
//!
//! Run with: cargo run --example docs_demo

use jfu::docs::get_docs;

fn main() {
    println!("=== Embedded Java Docs Demo ===\n");

    // Get the global docs index (this will trigger decompression and indexing on first access)
    let docs = get_docs();

    // Example 1: Look up a specific class
    println!("1️⃣  Looking up the 'String' class:");
    if let Some(class) = docs.get_class("String") {
        println!("   ✓ Found class: {}", class.name);
        println!("   ✓ Methods: {}", class.methods.len());
    }
    println!();

        println!("   ✓ Package description: {}", package.description);
    }
    println!();

    // Example 3: Look up methods by name across all classes
    println!("3️⃣  Finding all 'toString' methods:");
    let to_string_methods = docs.get_methods("toString");
    println!(
        "   ✓ Found {} classes with 'toString' method",
        to_string_methods.len()
    );
    for (package, class, method) in to_string_methods.iter().take(5) {
        println!("      • {}.{}.{}", package.package, class.name, method.name);
    }
    if to_string_methods.len() > 5 {
        println!("      ... and {} more", to_string_methods.len() - 5);
    }
    println!();

    // Example 4: Look up a specific method in a specific class
    println!("4️⃣  Looking up StringBuilder.append:");
    if let Some(method) = docs.get_class_method("StringBuilder", "append") {
        println!("   ✓ Method: {}", method.name);
        println!("   ✓ Overloads: {}", method.overloads.len());
        for (i, overload) in method.overloads.iter().take(3).enumerate() {
            println!("      [{}] {}", i + 1, overload.signature);
        }
        if method.overloads.len() > 3 {
            println!(
                "      ... and {} more overloads",
                method.overloads.len() - 3
            );
        }
    }
    println!();

    // Example 5: Search for classes by partial name
    println!("5️⃣  Searching for classes containing 'Stream':");
    let stream_classes = docs.search_classes("Stream");
    println!("   ✓ Found {} matches", stream_classes.len());
    for (fqn, class) in stream_classes.iter().take(5) {
        println!("      • {} ({} methods)", fqn, class.methods.len());
    }
    if stream_classes.len() > 5 {
        println!("      ... and {} more", stream_classes.len() - 5);
    }
    println!();

        println!("   ✓ Description: {}", package.description);
        println!("   ✓ Classes: {}", package.classes.len());
        for class in package.classes.iter().take(5) {
            println!("      • {} ({} methods)", class.name, class.methods.len());
        }
        if package.classes.len() > 5 {
            println!("      ... and {} more", package.classes.len() - 5);
        }
    }
    println!();

    // Example 7: Look up method details including description
    println!("7️⃣  Detailed method lookup - String.substring:");
    if let Some(method) = docs.get_class_method("String", "substring") {
        for (i, overload) in method.overloads.iter().enumerate() {
            println!("   Overload {}:", i + 1);
            println!("   Signature: {}", overload.signature);
            
                    }
                }
            }
            if overload.deprecated {
                println!("   ⚠️  DEPRECATED");
            }
            println!();
        }
    }

    // Example 8: Statistics
    println!("8️⃣  Documentation statistics:");
    println!("   ✓ Total packages: {}", docs.docs.packages.len());
    println!("   ✓ Total classes indexed: {}", docs.classes.len());
    println!("   ✓ Total method names indexed: {}", docs.methods.len());

    // Calculate total methods across all classes
    let total_methods: usize = docs
        .docs
        .packages
        .iter()
        .flat_map(|p| &p.classes)
        .map(|c| c.methods.len())
        .sum();
    println!("   ✓ Total methods: {}", total_methods);

    println!("\n=== Demo Complete ===");
}
