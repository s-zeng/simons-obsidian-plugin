use rust::{calculate_fibonacci, generate_demo_message, generate_greeting, process_editor_text};

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
    let results = vec![(0, calculate_fibonacci(0)), (1, calculate_fibonacci(1))];
    let snapshot = serde_json::to_string_pretty(&results).expect("Failed to serialize");
    insta::assert_snapshot!(
        snapshot,
        @r#"[
  [
    0,
    0
  ],
  [
    1,
    1
  ]
]"#
    );
}

#[test]
fn test_calculate_fibonacci_small_values() {
    let results = vec![
        (2, calculate_fibonacci(2)),
        (3, calculate_fibonacci(3)),
        (4, calculate_fibonacci(4)),
        (5, calculate_fibonacci(5)),
    ];
    let snapshot = serde_json::to_string_pretty(&results).expect("Failed to serialize");
    insta::assert_snapshot!(
        snapshot,
        @r#"[
  [
    2,
    1
  ],
  [
    3,
    2
  ],
  [
    4,
    3
  ],
  [
    5,
    5
  ]
]"#
    );
}

#[test]
fn test_calculate_fibonacci_medium_values() {
    let results = vec![
        (10, calculate_fibonacci(10)),
        (15, calculate_fibonacci(15)),
        (20, calculate_fibonacci(20)),
    ];
    let snapshot = serde_json::to_string_pretty(&results).expect("Failed to serialize");
    insta::assert_snapshot!(
        snapshot,
        @r#"[
  [
    10,
    55
  ],
  [
    15,
    610
  ],
  [
    20,
    6765
  ]
]"#
    );
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
