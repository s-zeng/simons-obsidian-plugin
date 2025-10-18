use wasm_bindgen::prelude::*;

/// Process editor text by transforming it.
///
/// # Arguments
/// * `selection` - The selected text to process
///
/// # Returns
/// Processed text with formatting
#[wasm_bindgen]
#[must_use]
pub fn process_editor_text(selection: &str) -> String {
    // For now, simple processing - can be extended with complex logic
    format!("Sample Editor Command\nProcessed: {selection}")
}

/// Generate a greeting message.
///
/// # Arguments
/// * `name` - Name to greet
///
/// # Returns
/// A personalized greeting
#[wasm_bindgen]
#[must_use]
pub fn generate_greeting(name: &str) -> String {
    format!("Hello, {name} from Rust!")
}

/// Calculate Fibonacci number (optimized iterative algorithm).
///
/// # Arguments
/// * `n` - The position in the Fibonacci sequence
///
/// # Returns
/// The nth Fibonacci number
#[wasm_bindgen]
#[must_use]
pub fn calculate_fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let mut a = 0;
            let mut b = 1;
            for _ in 2..=n {
                let temp = a + b;
                a = b;
                b = temp;
            }
            b
        },
    }
}

/// Generate demo message directly - all logic in Rust, no JSON round-trip.
///
/// # Arguments
/// * `name` - Name for greeting
/// * `a` - First number to add
/// * `b` - Second number to add
/// * `fib_n` - Fibonacci position to calculate
///
/// # Returns
/// A formatted message with greeting, sum, and Fibonacci result
#[wasm_bindgen]
#[must_use]
pub fn generate_demo_message(name: &str, a: i32, b: i32, fib_n: u32) -> String {
    let greeting = generate_greeting(name);
    let sum = a + b;
    let fib = calculate_fibonacci(fib_n);

    format!("{greeting}\nSum: {sum}\nFibonacci({fib_n}): {fib}")
}
