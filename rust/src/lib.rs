use wasm_bindgen::prelude::*;

// Module declarations
mod settings;
mod commands;
mod utils;

// Re-export all public functions from modules
pub use settings::*;
pub use commands::*;
pub use utils::*;

// Keep legacy functions for backwards compatibility (can be removed later)
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    generate_greeting(name)
}

#[wasm_bindgen]
pub fn fibonacci(n: u32) -> u32 {
    calculate_fibonacci(n)
}
