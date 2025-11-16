//! JFU - Java Fast Utility
//!
//! A fast, incremental build tool for Java with embedded API documentation.

pub mod docs;

// Re-export commonly used types
pub use docs::{Class, Method, Package, get_docs};
