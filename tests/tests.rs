use kalk_rs::*;
use std::collections::HashMap;
use std::f64::consts;
// A helper function to easily get the number value from the stack
fn get_number_at_top(stack: &Vec<StackItem>) -> f64 {
    match stack.last() {
        Some(StackItem::Number(val)) => *val,
        _ => panic!("Stack top is not a number or stack is empty"),
    }
}

#[test]
fn test_exp_function() {
    let mut stack = Vec::new();
    let mut storage = HashMap::new();
    let mut last_answer = None;

    // 1 exp = e^1 = e (approx 2.71828)
    stack.push(StackItem::Number(1.0));
    assert!(process_token(&mut stack, "exp", &mut last_answer, &mut storage).is_ok());
    assert!((get_number_at_top(&stack) - consts::E).abs() < 1e-15);

    // 0 exp = e^0 = 1.0
    stack.push(StackItem::Number(0.0));
    assert!(process_token(&mut stack, "exp", &mut last_answer, &mut storage).is_ok());
    assert!((get_number_at_top(&stack) - 1.0).abs() < 1e-15);
}

#[test]
fn test_log_function() {
    let mut stack = Vec::new();
    let mut storage = HashMap::new();
    let mut last_answer = None;

    // 100 10 log = log_10(100) = 2.0
    stack.push(StackItem::Number(100.0)); // x
    stack.push(StackItem::Number(10.0)); // base
    assert!(process_token(&mut stack, "log", &mut last_answer, &mut storage).is_ok());
    assert!((get_number_at_top(&stack) - 2.0).abs() < 1e-15);

    // 8 2 log = log_2(8) = 3.0
    stack.push(StackItem::Number(8.0)); // x
    stack.push(StackItem::Number(2.0)); // base
    assert!(process_token(&mut stack, "log", &mut last_answer, &mut storage).is_ok());
    assert!((get_number_at_top(&stack) - 3.0).abs() < 1e-15);

    // e e log = log_e(e) = 1.0
    stack.push(StackItem::Number(consts::E)); // x
    stack.push(StackItem::Number(consts::E)); // base
    assert!(process_token(&mut stack, "log", &mut last_answer, &mut storage).is_ok());
    assert!((get_number_at_top(&stack) - 1.0).abs() < 1e-15);
}

// --- EXISTING TESTS (Ensuring no regressions) ---

// Test: Basic Arithmetic
#[test]
fn test_basic_arithmetic() {
    let mut stack = vec![StackItem::Number(5.0), StackItem::Number(3.0)];
    let mut storage = HashMap::new();
    let mut last_answer = None;

    // 5 3 + = 8
    assert!(process_token(&mut stack, "+", &mut last_answer, &mut storage).is_ok());
    assert_eq!(get_number_at_top(&stack), 8.0);

    // 8 4 * = 32
    stack.push(StackItem::Number(4.0));
    assert!(process_token(&mut stack, "*", &mut last_answer, &mut storage).is_ok());
    assert_eq!(get_number_at_top(&stack), 32.0);
}

// Test: Unary and Constants
#[test]
fn test_unary_and_constants() {
    let mut stack = Vec::new();
    let mut storage = HashMap::new();
    let mut last_answer = None;

    // sqrt(9) = 3
    stack.push(StackItem::Number(9.0));
    assert!(process_token(&mut stack, "sqrt", &mut last_answer, &mut storage).is_ok());
    assert_eq!(get_number_at_top(&stack), 3.0);

    // pi
    assert!(process_token(&mut stack, "pi", &mut last_answer, &mut storage).is_ok());
    assert!((get_number_at_top(&stack) - 3.14159).abs() < 0.0001);
}

// Test Trigonometric Functions
#[test]
fn test_trig_functions() {
    let mut stack = Vec::new();
    let mut storage = HashMap::new();
    let mut last_answer = None;

    // pi sin should be very close to 0
    stack.push(StackItem::Number(consts::PI));
    assert!(process_token(&mut stack, "sin", &mut last_answer, &mut storage).is_ok());
    // Use a small epsilon for float comparison (sin(pi) is mathematically 0)
    assert!((get_number_at_top(&stack)).abs() < 1e-15);

    // pi cos should be -1
    stack.push(StackItem::Number(consts::PI));
    assert!(process_token(&mut stack, "cos", &mut last_answer, &mut storage).is_ok());
    assert!((get_number_at_top(&stack) - (-1.0)).abs() < 1e-15);

    // pi 4 / tan should be 1
    // Clear stack and push pi/4
    stack.clear();
    stack.push(StackItem::Number(consts::PI / 4.0));
    assert!(process_token(&mut stack, "tan", &mut last_answer, &mut storage).is_ok());
    assert!((get_number_at_top(&stack) - 1.0).abs() < 1e-15);
}

// Test: Swap and Clear
#[test]
fn test_stack_manipulation() {
    let mut stack = vec![
        StackItem::Number(1.0),
        StackItem::Number(2.0),
        StackItem::Number(3.0),
    ];
    let mut storage = HashMap::new();
    let mut last_answer = None;

    // 1 2 3 <> -> 1 3 2
    assert!(process_token(&mut stack, "<>", &mut last_answer, &mut storage).is_ok());
    assert_eq!(stack.len(), 3);

    let swapped_top = match &stack[2] {
        StackItem::Number(v) => *v,
        _ => 0.0,
    };
    let swapped_middle = match &stack[1] {
        StackItem::Number(v) => *v,
        _ => 0.0,
    };

    assert_eq!(swapped_top, 2.0);
    assert_eq!(swapped_middle, 3.0);

    // Clear stack
    assert!(process_token(&mut stack, "c", &mut last_answer, &mut storage).is_ok());
    assert!(stack.is_empty());
}

// Test: Store and Recall with Keys
#[test]
fn test_storage_rcl() {
    let mut stack = Vec::new();
    let mut storage = HashMap::new();
    let mut last_answer = None;

    // 100 "rate" sto
    stack.push(StackItem::Number(100.0));
    assert!(process_token(&mut stack, "\"rate\"", &mut last_answer, &mut storage).is_ok());
    assert!(process_token(&mut stack, "sto", &mut last_answer, &mut storage).is_ok());

    // Check storage map
    assert_eq!(*storage.get("rate").unwrap(), 100.0);
    assert!(stack.is_empty());

    // "rate" rcl
    assert!(process_token(&mut stack, "\"rate\"", &mut last_answer, &mut storage).is_ok());
    assert!(process_token(&mut stack, "rcl", &mut last_answer, &mut storage).is_ok());

    // Check stack after recall
    assert_eq!(get_number_at_top(&stack), 100.0);
}

// Test: Input Parsing (Persian/Commas)
#[test]
fn test_input_parsing() {
    let mut stack = Vec::new();
    let mut storage = HashMap::new();
    let mut last_answer = None;

    // Parse with commas
    assert!(process_token(&mut stack, "1,234.5", &mut last_answer, &mut storage).is_ok());
    assert_eq!(get_number_at_top(&stack), 1234.5);

    // Parse Persian digits
    assert!(process_token(&mut stack, "۱۲۳", &mut last_answer, &mut storage).is_ok());
    assert_eq!(get_number_at_top(&stack), 123.0);

    // Parse Persian digits with commas (should fail if comma isn't stripped, but works here)
    assert!(process_token(&mut stack, "۱,۲۳۴", &mut last_answer, &mut storage).is_ok());
    assert_eq!(get_number_at_top(&stack), 1234.0);
}

#[test]
fn test_input_comment_stripping() {
    let input_with_comment = "10 5 + # This is a comment about the sum";

    // The key logic from main() implemented here:
    let cleaned_input = input_with_comment
        .trim()
        .splitn(2, '#')
        .next()
        .unwrap_or("")
        .trim();

    // The tokens should only contain the calculator input
    assert_eq!(cleaned_input, "10 5 +");

    // Test a line that is only a comment
    let only_comment = "# Ignore this line";
    let cleaned_only_comment = only_comment
        .trim()
        .splitn(2, '#')
        .next()
        .unwrap_or("")
        .trim();

    assert_eq!(cleaned_only_comment, "");
}

#[test]
fn test_standard_arabic_parsing() {
    let mut stack = Vec::new();
    let mut storage = HashMap::new();
    let mut last_answer = None;
    let arabic_pi = "٣٫١٤١٥٩٢٦٥٣٥٨";

    assert!(process_token(&mut stack, arabic_pi, &mut last_answer, &mut storage).is_ok());
    assert!((get_number_at_top(&stack) - 3.14159265358).abs() < 1e-10);

    // Test with thousands separator
    // Original token: "١٬٠٠٠٫٥" (1,000.5)
    let arabic_thousand = "١٬٠٠٠٫٥";

    assert!(process_token(&mut stack, arabic_thousand, &mut last_answer, &mut storage).is_ok());
    assert_eq!(get_number_at_top(&stack), 1000.5);
}

#[test]
fn test_percent_change() {
    let mut stack = Vec::new();
    let mut storage = HashMap::new();
    let mut last_answer = None;

    // 25 50 %% = 100.0% increase
    stack.push(StackItem::Number(25.0));
    stack.push(StackItem::Number(50.0));
    assert!(process_token(&mut stack, "%%", &mut last_answer, &mut storage).is_ok());
    assert_eq!(get_number_at_top(&stack), 100.0); // (50 - 25) / 25 * 100 = 100.0

    // 100 75 %% = -25.0% decrease
    stack.push(StackItem::Number(100.0));
    stack.push(StackItem::Number(75.0));
    assert!(process_token(&mut stack, "%%", &mut last_answer, &mut storage).is_ok());
    assert_eq!(get_number_at_top(&stack), -25.0); // (75 - 100) / 100 * 100 = -25.0
}

#[test]
fn test_modulus() {
    let mut stack = Vec::new();
    let mut storage = HashMap::new();
    let mut last_answer = None;

    // 10 3 % = 1.0 (10 mod 3)
    stack.push(StackItem::Number(10.0));
    stack.push(StackItem::Number(3.0));
    assert!(process_token(&mut stack, "%", &mut last_answer, &mut storage).is_ok());
    assert_eq!(get_number_at_top(&stack), 1.0);

    // -10 3 % = 2.0 (Euclidean remainder: -10 = 3*(-4) + 2)
    stack.push(StackItem::Number(-10.0));
    stack.push(StackItem::Number(3.0));
    assert!(process_token(&mut stack, "%", &mut last_answer, &mut storage).is_ok());
    assert_eq!(get_number_at_top(&stack), 2.0);
}

#[test]
fn test_hex_display() {
    let mut stack = Vec::new();
    let mut storage = HashMap::new();
    let mut last_answer = None;

    stack.clear();

    // --- Part 1: Positive Number ---
    // Push 255.99 (should truncate to 255 -> 0xFF)
    stack.push(StackItem::Number(255.99)); // Stack size is now 1.

    // Execute 'hex'. Stack size should remain 1.
    assert!(process_token(&mut stack, "hex", &mut last_answer, &mut storage).is_ok());

    // Verify Stack Integrity: The original number should still be on the stack.
    assert_eq!(stack.len(), 1);
    assert_eq!(get_number_at_top(&stack), 255.99);

    // --- Part 2: Negative Number ---
    // Push -42.1
    stack.push(StackItem::Number(-42.1)); // Stack size is now 2.

    // Execute 'hex'. Stack size should remain 2.
    assert!(process_token(&mut stack, "hex", &mut last_answer, &mut storage).is_ok());

    // Verify Stack Integrity again.
    assert_eq!(stack.len(), 2);
    assert_eq!(get_number_at_top(&stack), -42.1);
}

// --- Trigonometric and Inverse Trigonometric Tests ---

#[test]
fn test_acos_function() {
    let mut stack = Vec::new();
    let mut storage = HashMap::new();
    let mut last_answer = None;

    // 1 acos = acos(1) = 0
    stack.push(StackItem::Number(1.0));
    assert!(process_token(&mut stack, "acos", &mut last_answer, &mut storage).is_ok());
    assert!((get_number_at_top(&stack)).abs() < 1e-15); // Result is 0.0

    // 0 acos = acos(0) = pi/2
    stack.push(StackItem::Number(0.0));
    assert!(process_token(&mut stack, "acos", &mut last_answer, &mut storage).is_ok());
    assert!((get_number_at_top(&stack) - consts::FRAC_PI_2).abs() < 1e-15);
}

#[test]
fn test_asin_function() {
    let mut stack = Vec::new();
    let mut storage = HashMap::new();
    let mut last_answer = None;

    // 1 asin = asin(1) = pi/2
    stack.push(StackItem::Number(1.0));
    assert!(process_token(&mut stack, "asin", &mut last_answer, &mut storage).is_ok());
    assert!((get_number_at_top(&stack) - consts::FRAC_PI_2).abs() < 1e-15);

    // -1 asin = asin(-1) = -pi/2
    stack.push(StackItem::Number(-1.0));
    assert!(process_token(&mut stack, "asin", &mut last_answer, &mut storage).is_ok());
    assert!((get_number_at_top(&stack) - (-consts::FRAC_PI_2)).abs() < 1e-15);
}

#[test]
fn test_atan_function() {
    let mut stack = Vec::new();
    let mut storage = HashMap::new();
    let mut last_answer = None;

    // 1 atan = atan(1) = pi/4
    stack.push(StackItem::Number(1.0));
    assert!(process_token(&mut stack, "atan", &mut last_answer, &mut storage).is_ok());
    assert!((get_number_at_top(&stack) - consts::FRAC_PI_4).abs() < 1e-15);
}

#[test]
fn test_atan2_function() {
    let mut stack = Vec::new();
    let mut storage = HashMap::new();
    let mut last_answer = None;

    // y=1, x=1: 1 1 atan2 = pi/4 (45 degrees)
    stack.push(StackItem::Number(1.0)); // y (a)
    stack.push(StackItem::Number(1.0)); // x (b)
    assert!(process_token(&mut stack, "atan2", &mut last_answer, &mut storage).is_ok());
    assert!((get_number_at_top(&stack) - consts::FRAC_PI_4).abs() < 1e-15);
}

#[test]
fn test_factorial() {
    let mut stack = Vec::new();
    let mut storage = HashMap::new();
    let mut last_answer = None;

    // 5 ! = 120.0
    stack.push(StackItem::Number(5.0));
    assert!(process_token(&mut stack, "!", &mut last_answer, &mut storage).is_ok());
    assert_eq!(get_number_at_top(&stack), 120.0);

    // 0 ! = 1.0
    stack.push(StackItem::Number(0.0));
    assert!(process_token(&mut stack, "!", &mut last_answer, &mut storage).is_ok());
    assert_eq!(get_number_at_top(&stack), 1.0);

    // 4.9 ! = 120.0 (rounds to 5)
    stack.push(StackItem::Number(4.9));
    assert!(process_token(&mut stack, "!", &mut last_answer, &mut storage).is_ok());
    assert_eq!(get_number_at_top(&stack), 120.0);
}

#[test]
fn test_permutations() {
    let mut stack = Vec::new();
    let mut storage = HashMap::new();
    let mut last_answer = None;

    // 5 3 P = P(5, 3) = 60.0 (n=5, k=3)
    stack.push(StackItem::Number(5.0));
    stack.push(StackItem::Number(3.0));
    assert!(process_token(&mut stack, "P", &mut last_answer, &mut storage).is_ok());
    assert_eq!(get_number_at_top(&stack), 60.0);
    stack.clear();

    // --- Error Tests ---

    // 3 5 P (Error: n < k)
    stack.push(StackItem::Number(3.0));
    stack.push(StackItem::Number(5.0));
    assert!(process_token(&mut stack, "P", &mut last_answer, &mut storage).is_err());
    // Stack should contain the original [3.0, 5.0]
    assert_eq!(stack.len(), 2);
    assert_eq!(get_number_at_top(&stack), 5.0); // k is on top
    stack.clear();

    // -5 3 P (Error: n < 0)
    stack.push(StackItem::Number(-5.0));
    stack.push(StackItem::Number(3.0));
    assert!(process_token(&mut stack, "P", &mut last_answer, &mut storage).is_err());
    // Stack should contain the original [-5.0, 3.0]
    assert_eq!(stack.len(), 2);
    assert_eq!(get_number_at_top(&stack), 3.0);
}

#[test]
fn test_combinations() {
    let mut stack = Vec::new();
    let mut storage = HashMap::new();
    let mut last_answer = None;

    // 5 3 C = C(5, 3) = 10.0 (n=5, k=3)
    stack.push(StackItem::Number(5.0));
    stack.push(StackItem::Number(3.0));
    assert!(process_token(&mut stack, "C", &mut last_answer, &mut storage).is_ok());
    assert_eq!(get_number_at_top(&stack), 10.0);
    stack.clear();

    // --- Error Tests ---

    // 3 5 C (Error: n < k)
    stack.push(StackItem::Number(3.0));
    stack.push(StackItem::Number(5.0));
    assert!(process_token(&mut stack, "C", &mut last_answer, &mut storage).is_err());
    // Stack should contain the original [3.0, 5.0]
    assert_eq!(stack.len(), 2);
    assert_eq!(get_number_at_top(&stack), 5.0); // k is on top
}

#[test]
fn test_ceil_floor() {
    let mut stack = Vec::new();
    let mut storage = HashMap::new();
    let mut last_answer = None;

    // 1.1 ceil = 2.0
    stack.push(StackItem::Number(1.1));
    assert!(process_token(&mut stack, "ceil", &mut last_answer, &mut storage).is_ok());
    assert_eq!(get_number_at_top(&stack), 2.0);

    // -1.1 floor = -2.0
    stack.push(StackItem::Number(-1.1));
    assert!(process_token(&mut stack, "floor", &mut last_answer, &mut storage).is_ok());
    assert_eq!(get_number_at_top(&stack), -2.0);
}

#[test]
fn test_angle_conversions() {
    let mut stack = Vec::new();
    let mut storage = HashMap::new();
    let mut last_answer = None;

    // pi rad deg = 180.0
    stack.push(StackItem::Number(consts::PI));
    assert!(process_token(&mut stack, "deg", &mut last_answer, &mut storage).is_ok());
    assert!((get_number_at_top(&stack) - 180.0).abs() < 1e-10);

    // 180 deg rad = pi
    stack.push(StackItem::Number(180.0));
    assert!(process_token(&mut stack, "rad", &mut last_answer, &mut storage).is_ok());
    assert!((get_number_at_top(&stack) - consts::PI).abs() < 1e-10);
}
