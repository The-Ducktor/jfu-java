use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, fs, path::Path};

use crate::graph::Node;

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheEntry {
    pub hash: String,
    pub class_path: String,
}

pub type Cache = HashMap<String, CacheEntry>;

pub fn load_cache(cache_path: &Path) -> Cache {
    if cache_path.exists() {
        let content = fs::read_to_string(cache_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    }
}

pub fn save_cache(cache_path: &Path, cache: &Cache) {
    let json = serde_json::to_string_pretty(cache).unwrap();
    fs::write(cache_path, json).unwrap_or_else(|e| {
        use colored::*;
        eprintln!("{} Failed to save cache: {}", "⚠️".yellow(), e);
    });
}

pub fn compute_hash(path: &Path) -> String {
    let content = fs::read(path).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}

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
