// src/loader/mod.rs

/// The `error_handling` module provides error handling functionality for the YAML loader.
pub mod error_handling;
/// The `memory` module provides memory management functionality for the YAML loader.
pub mod memory;

/// The `loader` module is responsible for parsing YAML documents.
/// It handles various YAML events and constructs the corresponding YAML document structure.
pub mod parsing;

pub use error_handling::*;
pub use memory::*;
pub use parsing::*;
