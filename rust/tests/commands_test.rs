use rust::{
    process_editor_text,
    generate_greeting,
    calculate_fibonacci,
    generate_demo_message,
};

#[test]
fn test_process_editor_text_simple() {
    let input = "hello world";
    let result = process_editor_text(input);
    insta::assert_snapshot!(result, @r"Sample Editor Command
Processed: hello world");
}

#[test]
fn test_process_editor_text_empty() {
    let input = "";
    let result = process_editor_text(input);
    insta::assert_snapshot!(result, @r"Sample Editor Command
Processed: ");
}

#[test]
fn test_process_editor_text_multiline() {
    let input = "line1\nline2\nline3";
    let result = process_editor_text(input);
    insta::assert_snapshot!(result, @r"Sample Editor Command
Processed: line1
line2
line3");
}

#[test]
fn test_generate_greeting() {
    let result = generate_greeting("Alice");
    insta::assert_snapshot!(result, @"Hello, Alice from Rust!");
}

#[test]
fn test_generate_greeting_empty_name() {
    let result = generate_greeting("");
    insta::assert_snapshot!(result, @"Hello,  from Rust!");
}

#[test]
fn test_calculate_fibonacci_base_cases() {
    insta::assert_snapshot!(calculate_fibonacci(0), @"0");
    insta::assert_snapshot!(calculate_fibonacci(1), @"1");
}

#[test]
fn test_calculate_fibonacci_small_values() {
    insta::assert_snapshot!(calculate_fibonacci(2), @"1");
    insta::assert_snapshot!(calculate_fibonacci(3), @"2");
    insta::assert_snapshot!(calculate_fibonacci(4), @"3");
    insta::assert_snapshot!(calculate_fibonacci(5), @"5");
}

#[test]
fn test_calculate_fibonacci_medium_values() {
    insta::assert_snapshot!(calculate_fibonacci(10), @"55");
    insta::assert_snapshot!(calculate_fibonacci(15), @"610");
    insta::assert_snapshot!(calculate_fibonacci(20), @"6765");
}

#[test]
fn test_generate_demo_message() {
    let result = generate_demo_message("TestUser", 5, 7, 10);
    insta::assert_snapshot!(result, @r"Hello, TestUser from Rust!
Sum: 12
Fibonacci(10): 55");
}

#[test]
fn test_generate_demo_message_zero_values() {
    let result = generate_demo_message("Zero", 0, 0, 0);
    insta::assert_snapshot!(result, @r"Hello, Zero from Rust!
Sum: 0
Fibonacci(0): 0");
}

#[test]
fn test_generate_demo_message_negative_sum() {
    let result = generate_demo_message("Negative", -5, 3, 1);
    insta::assert_snapshot!(result, @r"Hello, Negative from Rust!
Sum: -2
Fibonacci(1): 1");
}
