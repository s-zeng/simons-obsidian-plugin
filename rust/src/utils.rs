use wasm_bindgen::prelude::*;

/// Basic string utilities that can be extended for text processing.
/// Reverse a string by reversing its characters.
///
/// # Arguments
/// * `input` - The string to reverse
///
/// # Returns
/// The reversed string
#[wasm_bindgen]
#[must_use]
pub fn reverse_string(input: &str) -> String {
    input.chars().rev().collect()
}

/// Count the number of words in a string.
///
/// # Arguments
/// * `input` - The string to count words in
///
/// # Returns
/// The number of whitespace-separated words
#[wasm_bindgen]
#[must_use]
pub fn word_count(input: &str) -> usize {
    input.split_whitespace().count()
}

/// Convert a string to title case.
///
/// # Arguments
/// * `input` - The string to convert
///
/// # Returns
/// String with each word capitalized
#[wasm_bindgen]
#[must_use]
pub fn to_title_case(input: &str) -> String {
    input
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// Placeholder for future numeric/ML algorithms

/// Add two integers.
///
/// # Arguments
/// * `a` - First integer
/// * `b` - Second integer
///
/// # Returns
/// The sum of a and b
#[wasm_bindgen]
#[must_use]
#[allow(clippy::missing_const_for_fn)] // wasm_bindgen doesn't support const fn
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Multiply two integers.
///
/// # Arguments
/// * `a` - First integer
/// * `b` - Second integer
///
/// # Returns
/// The product of a and b
#[wasm_bindgen]
#[must_use]
#[allow(clippy::missing_const_for_fn)] // wasm_bindgen doesn't support const fn
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

// Future: Add linear algebra, statistics, ML utilities here
// Examples:
// - Vector operations
// - Matrix operations
// - Statistical functions (mean, median, std dev)
// - Simple ML algorithms (k-means, linear regression, etc.)
