use wasm_bindgen::prelude::*;

/// Basic string utilities that can be extended for text processing

#[wasm_bindgen]
pub fn reverse_string(input: &str) -> String {
    input.chars().rev().collect()
}

#[wasm_bindgen]
pub fn word_count(input: &str) -> usize {
    input.split_whitespace().count()
}

#[wasm_bindgen]
pub fn to_title_case(input: &str) -> String {
    input
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// Placeholder for future numeric/ML algorithms
#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[wasm_bindgen]
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

// Future: Add linear algebra, statistics, ML utilities here
// Examples:
// - Vector operations
// - Matrix operations
// - Statistical functions (mean, median, std dev)
// - Simple ML algorithms (k-means, linear regression, etc.)
