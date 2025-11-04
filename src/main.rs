use std::collections::HashMap;
use std::f64::consts;
use std::io::{self, Write};
use thousands::Separable;

/// Represents an item that can be placed on the RPN stack.
/// It can be a floating-point number or a string key for storage.
#[derive(Debug, Clone)]
enum StackItem {
    Number(f64),
    Key(String),
}

/// Helper to convert Persian (Eastern) and standard Arabic (Western) digits
/// and separators to ASCII digits and standard separators.
fn unicode_to_ascii(c: char) -> char {
    match c {
        // 1. Persian (Extended Arabic-Indic) Digits: ۰ to ۹ (U+06F0 to U+06F9)
        '۰' => '0',
        '۱' => '1',
        '۲' => '2',
        '۳' => '3',
        '۴' => '4',
        '۵' => '5',
        '۶' => '6',
        '۷' => '7',
        '۸' => '8',
        '۹' => '9',

        // 2. Standard Arabic (Arabic-Indic) Digits: ٠ to ٩ (U+0660 to U+0669)
        '٠' => '0',
        '١' => '1',
        '٢' => '2',
        '٣' => '3',
        '٤' => '4',
        '٥' => '5',
        '٦' => '6',
        '٧' => '7',
        '٨' => '8',
        '٩' => '9',

        // 3. Persian/Arabic Separators
        '٫' => '.', // Arabic Decimal Separator -> ASCII Period
        '٬' => ',', // Arabic Thousands Separator -> ASCII Comma

        _ => c, // Leave all other characters unchanged
    }
}

/// Unary function for single-operand operations (e.g., sqrt).
/// Pops one number, applies the function, and pushes the result.
fn unary_calculate<F>(stack: &mut Vec<StackItem>, op: F) -> Result<(), &'static str>
where
    F: Fn(f64) -> f64,
{
    // Pop the last item and ensure it's a Number
    let a = match stack.pop() {
        Some(StackItem::Number(val)) => val,
        _ => return Err("Unary operation requires one number on the stack"),
    };

    // Perform calculation and push the result
    stack.push(StackItem::Number(op(a)));
    Ok(())
}

/// Binary function for two-operand operations (e.g., +, -, *, /).
/// Pops two numbers (a and b), applies the function (a op b), and pushes the result.
fn calculate<F>(stack: &mut Vec<StackItem>, op: F, _op_symbol: &str) -> Result<(), &'static str>
where
    F: Fn(f64, f64) -> f64,
{
    // RPN needs two operands: pop the second-to-last (b) and last (a)
    let b = match stack.pop() {
        Some(StackItem::Number(val)) => val,
        _ => {
            return Err(
                "Binary operation requires two numbers on the stack (missing second operand)",
            );
        }
    };
    let a = match stack.pop() {
        Some(StackItem::Number(val)) => val,
        _ => {
            return Err(
                "Binary operation requires two numbers on the stack (missing first operand)",
            );
        }
    };

    // Perform the calculation and push the result
    stack.push(StackItem::Number(op(a, b)));
    Ok(())
}

/// Swaps the position of the last two number values on the stack.
fn swap(stack: &mut Vec<StackItem>) -> Result<(), &'static str> {
    if stack.len() < 2 {
        return Err("Not enough items on the stack to swap");
    }

    // Use the built-in Vec swap method based on indices
    let idx_a = stack.len() - 1;
    let idx_b = stack.len() - 2;

    // We swap the StackItem enum variants directly
    stack.swap(idx_a, idx_b);

    Ok(())
}

/// The core function to process a single input token.
fn process_token(
    stack: &mut Vec<StackItem>,
    token: &str,
    last_answer: &mut Option<f64>,
    storage: &mut HashMap<String, f64>,
) -> Result<(), &'static str> {
    // 1. Check for Quoted String Key
    if token.starts_with('"') && token.ends_with('"') && token.len() > 1 {
        let key = token.trim_matches('"').to_string();
        stack.push(StackItem::Key(key));
        return Ok(());
    }

    // 2. Handle Numeric Input
    // First, convert Persian digits to Arabic and remove thousand separators (commas)
    let cleaned_token: String = token
        .chars()
        .map(unicode_to_ascii)
        .filter(|c| *c != ',')
        .collect();

    if let Ok(num) = cleaned_token.parse::<f64>() {
        stack.push(StackItem::Number(num));
        return Ok(());
    }

    // 3. Handle Commands and Operators
    match token {
        // Arithmetic Operators (Binary)
        "+" => calculate(stack, |a, b| a + b, "+"),
        "-" => calculate(stack, |a, b| a - b, "-"),
        "*" | "x" => calculate(stack, |a, b| a * b, "*"),
        "/" => calculate(stack, |a, b| a / b, "/"),
        "%" => calculate(stack, |a, b| a.rem_euclid(b), "%"),
        "**" => calculate(stack, |a, b| a.powf(b), "**"),
        "%%" => calculate(stack, |a, b| (b - a) / a * 100.0, "%%"),

        // Unary Operators
        "sqrt" => unary_calculate(stack, f64::sqrt),

        // Constants
        "pi" => {
            stack.push(StackItem::Number(consts::PI));
            Ok(())
        }
        "e" => {
            stack.push(StackItem::Number(consts::E));
            Ok(())
        }

        // Stack Manipulation
        "<>" => swap(stack), // Swap
        "c" => {
            stack.clear();
            Ok(())
        } // Clear stack

        // Last Answer (a)
        "a" => {
            if let Some(val) = *last_answer {
                stack.push(StackItem::Number(val));
                Ok(())
            } else {
                Err("No previous answer available ('a' is empty)")
            }
        }

        // Store (sto)
        // Usage: <value> "key" sto
        "sto" => {
            // Key is the top item, Value is the second item
            let key = match stack.pop() {
                Some(StackItem::Key(k)) => k,
                _ => return Err("STO requires a string key (e.g., \"rate\") as the last item"),
            };
            let val = match stack.pop() {
                Some(StackItem::Number(v)) => v,
                _ => return Err("STO requires a number value before the key"),
            };

            storage.insert(key, val);
            Ok(())
        }

        // Recall (rcl)
        // Usage: "key" rcl
        "rcl" => {
            // Key is the top item
            let key = match stack.pop() {
                Some(StackItem::Key(k)) => k,
                _ => return Err("RCL requires a string key (e.g., \"rate\") as the last item"),
            };

            if let Some(&val) = storage.get(&key) {
                stack.push(StackItem::Number(val));
                Ok(())
            } else {
                Err("Storage key not found")
            }
        }

        _ => Err("Unrecognized token or operator"),
    }
}

fn main() {
    let mut stack: Vec<StackItem> = Vec::new(); // Refactored to Vec<StackItem>
    let mut last_answer: Option<f64> = None;
    let mut storage: HashMap<String, f64> = HashMap::new();

    println!("Welcome to kalk-rs (RPN Calculator). Type 'exit' to quit.");
    println!("Supported: +, -, *, /, **, %%, %, sqrt, pi, e, <>, c, a, \"key\" sto, \"key\" rcl.");

    loop {
        // Manually format the stack for a cleaner look.
        let display_content: Vec<String> = stack
            .iter()
            .map(|item| {
                match item {
                    StackItem::Number(val) => val.separate_with_commas(),
                    // Display keys surrounded by their quotes
                    StackItem::Key(key) => format!("\"{}\"", key),
                }
            })
            .collect();

        // Join the items and wrap in square brackets
        let display_string = format!("[{}]", display_content.join(", "));

        // Display the current stack state using the new display_string
        print!("Stack: {}\n> ", display_string);

        io::stdout().flush().unwrap();

        // Read user input
        let mut input = String::new();
        if let Err(e) = io::stdin().read_line(&mut input) {
            eprintln!("I/O Error: {}", e);
            continue;
        }

        // Check for comment marker (#) and strip the rest of the line
        let input = input.splitn(2, '#').next().unwrap_or("").trim();

        if input.eq_ignore_ascii_case("exit") {
            break;
        }

        // Process tokens
        let mut tokens = input.split_whitespace();
        let mut success = true;

        // Note: For string keys, tokens like "rate" need to be handled carefully
        // if they are part of a multi-token input line.
        // This simple split works if keys are separate, e.g., 10 "rate" sto
        while let Some(token) = tokens.next() {
            if let Err(e) = process_token(&mut stack, token, &mut last_answer, &mut storage) {
                eprintln!("Error: {}", e);
                // On error, clear the current input line's processing
                success = false;
                break;
            }
        }

        // Update Last Answer ONLY if the input line processed successfully
        if success {
            if let Some(StackItem::Number(result)) = stack.last() {
                last_answer = Some(*result);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A helper function to easily get the number value from the stack
    fn get_number_at_top(stack: &Vec<StackItem>) -> f64 {
        match stack.last() {
            Some(StackItem::Number(val)) => *val,
            _ => panic!("Stack top is not a number or stack is empty"),
        }
    }

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
}
