use std::collections::HashMap;
use std::f64::consts;
use std::io::{self, Write};

/// Represents an item that can be placed on the RPN stack.
/// It can be a floating-point number or a string key for storage.
#[derive(Debug, Clone)]
enum StackItem {
    Number(f64),
    Key(String),
}

/// Helper to convert Persian digits to Arabic (ASCII) digits.
fn persian_to_arabic(c: char) -> char {
    match c {
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
        _ => c, // Leave all other characters (., -, +, etc.) unchanged
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
        .map(persian_to_arabic)
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
        "**" => calculate(stack, |a, b| a.powf(b), "**"),

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
    println!("Supported: +, -, *, /, **, sqrt, pi, e, <>, c, a, \"key\" sto, \"key\" rcl.");

    loop {
        // Display the current stack state
        print!("Stack: {:?}\n> ", stack);
        io::stdout().flush().unwrap();

        // Read user input
        let mut input = String::new();
        if let Err(e) = io::stdin().read_line(&mut input) {
            eprintln!("I/O Error: {}", e);
            continue;
        }

        let input = input.trim();

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
