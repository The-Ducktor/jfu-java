//! Embedded Java documentation module
//!
//! This module provides access to embedded, compressed Java API documentation.
//! The documentation is compressed at compile time and decompressed on first access,
//! with an in-memory index built for fast lookups.
//!
//! # Usage
//!
//! ```rust
//! use crate::docs::{get_docs, DocsIndex};
//!
//! // Get the global docs index
//! let docs = get_docs();
//!
//! // Look up a class
//! if let Some(class) = docs.get_class("String") {
//!     println!("Found class: {}", class.name);
//! }
//!
//! // Look up a method
//! let methods = docs.get_methods("println");
//! for (package, class, method) in methods {
//!     println!("{}::{}", class.name, method.name);
//! }
//! ```

mod embedded;
mod types;

pub use embedded::{get_docs, init_docs};
pub use types::{Class, Method, Package};
