//! Obsidian plugin powered by Rust and WebAssembly.
//!
//! This library provides high-performance computational functions for an Obsidian plugin,
//! leveraging Rust's safety and performance through WebAssembly compilation.

use wasm_bindgen::prelude::*;

// Module declarations
mod commands;
mod error;
mod settings;
mod utils;

// Re-export all public functions from modules
pub use commands::*;
pub use error::*;
pub use settings::*;
pub use utils::*;

/// Generate a greeting message (legacy compatibility function).
///
/// # Arguments
/// * `name` - The name to greet
///
/// # Returns
/// A greeting string
#[wasm_bindgen]
#[must_use]
pub fn greet(name: &str) -> String {
    generate_greeting(name)
}

/// Calculate the nth Fibonacci number (legacy compatibility function).
///
/// # Arguments
/// * `n` - The position in the Fibonacci sequence
///
/// # Returns
/// The Fibonacci number at position n
#[wasm_bindgen]
#[must_use]
pub fn fibonacci(n: u32) -> u32 {
    calculate_fibonacci(n)
}
