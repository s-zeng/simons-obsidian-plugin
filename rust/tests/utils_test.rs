use rust::{add, multiply, reverse_string, to_title_case, word_count};

#[test]
fn test_reverse_string_simple() {
    let result = reverse_string("hello");
    insta::assert_snapshot!(result, @"olleh");
}

#[test]
fn test_reverse_string_empty() {
    let result = reverse_string("");
    insta::assert_snapshot!(result, @"");
}

#[test]
fn test_reverse_string_palindrome() {
    let result = reverse_string("racecar");
    insta::assert_snapshot!(result, @"racecar");
}

#[test]
fn test_reverse_string_unicode() {
    let result = reverse_string("Hello, 世界");
    insta::assert_snapshot!(result, @"界世 ,olleH");
}

#[test]
fn test_word_count_simple() {
    let result = word_count("hello world");
    insta::assert_snapshot!(result, @"2");
}

#[test]
fn test_word_count_empty() {
    let result = word_count("");
    insta::assert_snapshot!(result, @"0");
}

#[test]
fn test_word_count_multiple_spaces() {
    let result = word_count("hello    world    test");
    insta::assert_snapshot!(result, @"3");
}

#[test]
fn test_word_count_leading_trailing_spaces() {
    let result = word_count("  hello world  ");
    insta::assert_snapshot!(result, @"2");
}

#[test]
fn test_word_count_single_word() {
    let result = word_count("hello");
    insta::assert_snapshot!(result, @"1");
}

#[test]
fn test_to_title_case_simple() {
    let result = to_title_case("hello world");
    insta::assert_snapshot!(result, @"Hello World");
}

#[test]
fn test_to_title_case_already_titled() {
    let result = to_title_case("Hello World");
    insta::assert_snapshot!(result, @"Hello World");
}

#[test]
fn test_to_title_case_all_caps() {
    let result = to_title_case("HELLO WORLD");
    insta::assert_snapshot!(result, @"Hello World");
}

#[test]
fn test_to_title_case_mixed() {
    let result = to_title_case("hElLo WoRlD");
    insta::assert_snapshot!(result, @"Hello World");
}

#[test]
fn test_to_title_case_empty() {
    let result = to_title_case("");
    insta::assert_snapshot!(result, @"");
}

#[test]
fn test_to_title_case_single_word() {
    let result = to_title_case("hello");
    insta::assert_snapshot!(result, @"Hello");
}

#[test]
fn test_add_positive() {
    let result = add(5, 7);
    insta::assert_snapshot!(result, @"12");
}

#[test]
fn test_add_negative() {
    let result = add(-5, -3);
    insta::assert_snapshot!(result, @"-8");
}

#[test]
fn test_add_mixed() {
    let result = add(-5, 10);
    insta::assert_snapshot!(result, @"5");
}

#[test]
fn test_add_zero() {
    let result = add(0, 0);
    insta::assert_snapshot!(result, @"0");
}

#[test]
fn test_multiply_positive() {
    let result = multiply(5, 7);
    insta::assert_snapshot!(result, @"35");
}

#[test]
fn test_multiply_negative() {
    let result = multiply(-5, -3);
    insta::assert_snapshot!(result, @"15");
}

#[test]
fn test_multiply_mixed() {
    let result = multiply(-5, 10);
    insta::assert_snapshot!(result, @"-50");
}

#[test]
fn test_multiply_by_zero() {
    let result = multiply(5, 0);
    insta::assert_snapshot!(result, @"0");
}
