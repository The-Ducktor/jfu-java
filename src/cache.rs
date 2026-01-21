//! Build cache management for incremental compilation.
//!
//! This module stores and checks file hashes to determine if recompilation is needed.
//! Files are only rebuilt if:
//! - They don't exist in the cache
//! - Their hash has changed since the last build
//! - The --force flag was used
//!
//! The cache is stored as JSON in `jfu-cache.json` (or configured path).
//! Uses xxHash for fast hashing (much faster than SHA-256 for build caching).

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};
use xxhash_rust::xxh64::xxh64;

use crate::graph::Node;

/// A single entry in the build cache containing a file's hash and output path.
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheEntry {
    /// xxHash64 hash of the Java source file content (as hex string)
    pub hash: String,
    /// Path to the compiled .class file
    pub class_path: String,
}

/// The complete build cache: filename -> cache entry
pub type Cache = HashMap<String, CacheEntry>;

/// Load the build cache from disk.
///
/// If the cache file doesn't exist or can't be parsed, returns an empty cache.
///
/// # Arguments
/// * `cache_path` - Path to the cache JSON file
///
/// # Returns
/// The parsed cache or an empty HashMap if loading fails
pub fn load_cache(cache_path: &Path) -> Cache {
    if cache_path.exists() {
        let content = fs::read_to_string(cache_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    }
}

/// Save the build cache to disk.
///
/// Writes the cache as pretty-printed JSON. Errors are logged but non-fatal.
///
/// # Arguments
/// * `cache_path` - Path to the cache JSON file
/// * `cache` - The cache to persist
pub fn save_cache(cache_path: &Path, cache: &Cache) {
    let json = serde_json::to_string_pretty(cache).unwrap();
    fs::write(cache_path, json).unwrap_or_else(|e| {
        use colored::*;
        eprintln!("{} Failed to save cache: {}", "⚠️".yellow(), e);
    });
}

/// Compute the xxHash64 hash of a file's contents.
///
/// xxHash64 is much faster than SHA-256 and sufficient for detecting
/// file changes in the build cache. Returns "0" if the file can't be read.
///
/// # Arguments
/// * `path` - Path to the file to hash
///
/// # Returns
/// A hex-encoded xxHash64 hash of the file contents
pub fn compute_hash(path: &Path) -> String {
    let content = fs::read(path).unwrap_or_default();
    let hash = xxh64(&content, 0);
    format!("{:x}", hash)
}

/// Check if a Java file needs to be recompiled.
///
/// A file needs rebuilding if:
/// - The force flag is set, or
/// - The .class file doesn't exist, or
/// - The file is not in the cache, or
/// - The file's hash has changed since the last build
///
/// # Arguments
/// * `node` - The source file node with path and name
/// * `cache` - The current build cache
/// * `out_dir` - Output directory where .class files are written
/// * `force` - If true, always rebuild regardless of cache status
///
/// # Returns
/// true if the file should be recompiled
pub fn needs_rebuild(node: &Node, cache: &Cache, out_dir: &Path, force: bool) -> bool {
    if force {
        return true;
    }

    let class_name = node.name.strip_suffix(".java").unwrap_or(&node.name);
    let class_path = out_dir.join(format!("{}.class", class_name));

    // If .class doesn't exist, rebuild
    if !class_path.exists() {
        return true;
    }

    // If not in cache, rebuild
    let Some(entry) = cache.get(&node.name) else {
        return true;
    };

    // If hash changed, rebuild
    let current_hash = compute_hash(&node.path);
    current_hash != entry.hash
}
