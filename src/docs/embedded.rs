use super::types::{DocsIndex, JavaDocs};
use flate2::read::GzDecoder;
use lazy_static::lazy_static;
use std::io::Read;

// Embed the compressed JSON at compile time
static COMPRESSED_DOCS: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/all-packages-methods.json.gz"));

lazy_static! {
    /// Global docs index, initialized on first access
    pub static ref DOCS: DocsIndex = load_docs();
}

/// Load and decompress the embedded docs, then build the index
fn load_docs() -> DocsIndex {
    let start = std::time::Instant::now();

    // Decompress the embedded data
    let mut decoder = GzDecoder::new(COMPRESSED_DOCS);
    let mut json_str = String::new();
    decoder
        .read_to_string(&mut json_str)
        .expect("Failed to decompress embedded Java docs");

    let decompress_time = start.elapsed();

    // Parse the JSON
    let parse_start = std::time::Instant::now();
    let docs: JavaDocs =
        serde_json::from_str(&json_str).expect("Failed to parse embedded Java docs JSON");
    let parse_time = parse_start.elapsed();

    // Build the index
    let index_start = std::time::Instant::now();
    let index = DocsIndex::build(docs);
    let index_time = index_start.elapsed();

    let total_time = start.elapsed();

    eprintln!("📚 Java docs loaded:");
    eprintln!("   Decompression: {:?}", decompress_time);
    eprintln!("   JSON parsing:  {:?}", parse_time);
    eprintln!("   Index build:   {:?}", index_time);
    eprintln!("   Total time:    {:?}", total_time);
    eprintln!("   Classes indexed: {}", index.classes.len());
    eprintln!("   Methods indexed: {}", index.methods.len());

    index
}

/// Get a reference to the global docs index
pub fn get_docs() -> &'static DocsIndex {
    &DOCS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docs_load() {
        let docs = get_docs();
        assert!(
            !docs.docs.packages.is_empty(),
            "Docs should contain packages"
        );
        assert!(!docs.classes.is_empty(), "Class index should not be empty");
        assert!(!docs.methods.is_empty(), "Method index should not be empty");
    }

    #[test]
    fn test_class_lookup() {
        let docs = get_docs();

        // Test looking up a common Java class
        let string_class = docs.get_class("String");
        assert!(string_class.is_some(), "Should find String class");

        // Test fully qualified name
        let fq_string = docs.get_class("java.lang.String");
        assert!(fq_string.is_some(), "Should find java.lang.String");
    }

    #[test]
    fn test_method_lookup() {
        let docs = get_docs();

        // Test looking up a common method
        let to_string_methods = docs.get_methods("toString");
        assert!(
            !to_string_methods.is_empty(),
            "Should find toString methods"
        );
    }

    #[test]
    fn test_class_method_lookup() {
        let docs = get_docs();

        // Test looking up a specific method in a specific class
        let append_method = docs.get_class_method("StringBuilder", "append");
        assert!(
            append_method.is_some(),
            "Should find StringBuilder.append method"
        );

        if let Some(method) = append_method {
            assert!(!method.overloads.is_empty(), "append should have overloads");
        }
    }

    #[test]
    fn test_package_lookup() {
        let docs = get_docs();

        let java_lang = docs.get_package("java.lang");
        assert!(java_lang.is_some(), "Should find java.lang package");

        if let Some(pkg) = java_lang {
            assert!(!pkg.classes.is_empty(), "java.lang should have classes");
        }
    }

    #[test]
    fn test_search() {
        let docs = get_docs();

        let results = docs.search_classes("Array");
        assert!(!results.is_empty(), "Should find classes matching 'Array'");
    }
}
