use phf::{Map, phf_map};
use std::collections::HashMap;
use std::f64::consts;
use std::io::{self, Write};
use thousands::Separable;

/// Type alias for a function that operates on one f64 and returns an f64.
type UnaryHandler = fn(f64) -> f64;
/// Type alias for a function that operates on two f64s and returns an f64.
type BinaryHandler = fn(f64, f64) -> f64;

// --- Helper Handler Definitions (for functions not available on f64::) ---

fn rad_to_deg(rad: f64) -> f64 {
    rad * 180.0 / consts::PI
}
fn deg_to_rad(deg: f64) -> f64 {
    deg * consts::PI / 180.0
}
fn log_op(a: f64, b: f64) -> f64 {
    a.log(b)
}
fn percent_change(a: f64, b: f64) -> f64 {
    (b - a) / a * 100.0
}
fn power_op(a: f64, b: f64) -> f64 {
    a.powf(b)
}
fn atan2_op(y: f64, x: f64) -> f64 {
    y.atan2(x)
} // RPN stack order requires Y X

// --- Centralized Operator Data Structures ---

/// Represents the execution logic for an operator.
/// This allows the static map to hold heterogeneous handler types.
enum OperatorAction {
    PushConstant(f64),
    Unary(UnaryHandler),
    Binary(BinaryHandler),
    /// For functions that require custom access to the stack, memory, or last answer.
    Special(&'static str),
}

/// The centralized, static map containing ALL operator information:
/// 1. Token (Key)
/// 2. Help Group (&'static str)
/// 3. Usage String (&'static str)
/// 4. Execution Action (OperatorAction enum)
const OPERATOR_DATA: Map<&'static str, (&'static str, &'static str, OperatorAction)> = phf_map! {
    // Arithmetic (Binary)
    "+" => ("Binary", "a b + | Addition (a + b)", OperatorAction::Binary(|a, b| a + b)),
    "-" => ("Binary", "a b - | Subtraction (a - b)", OperatorAction::Binary(|a, b| a - b)),
    "*" => ("Binary", "a b * | Multiplication (a * b)", OperatorAction::Binary(|a, b| a * b)),
    "/" => ("Binary", "a b / | Division (a / b)", OperatorAction::Binary(|a, b| a / b)),
    "**" => ("Binary", "a b ** | Power (a^b)", OperatorAction::Binary(power_op)),
    "%" => ("Binary", "a b % | Euclidean Remainder (a mod b)", OperatorAction::Binary(f64::rem_euclid)),
    "%%" => ("Binary", "a b %% | Percent Change ((b - a) / a * 100)", OperatorAction::Binary(percent_change)),
    "log" => ("Binary", "a b log | Logarithm (log_b(a))", OperatorAction::Binary(log_op)),
    "atan2" => ("Binary", "y x atan2 | Arc tangent of y/x (result in radians)", OperatorAction::Binary(atan2_op)),

    // Constants
    "pi" => ("Constant", "pi | Push the value of pi", OperatorAction::PushConstant(consts::PI)),
    "e" => ("Constant", "e | Push the value of Euler's number (e)", OperatorAction::PushConstant(consts::E)),

    // Unary/Trig/Rounding (Unary Handler)
    "sqrt" => ("Unary", "a sqrt | Square root", OperatorAction::Unary(f64::sqrt)),
    "sin" => ("Unary", "a sin | Sine (a in radians)", OperatorAction::Unary(f64::sin)),
    "cos" => ("Unary", "a cos | Cosine (a in radians)", OperatorAction::Unary(f64::cos)),
    "tan" => ("Unary", "a tan | Tangent (a in radians)", OperatorAction::Unary(f64::tan)),
    "acos" => ("Unary", "a acos | Arc cosine (result in radians)", OperatorAction::Unary(f64::acos)),
    "asin" => ("Unary", "a asin | Arc sine (result in radians)", OperatorAction::Unary(f64::asin)),
    "atan" => ("Unary", "a atan | Arc tangent (result in radians)", OperatorAction::Unary(f64::atan)),
    "exp" => ("Unary", "a exp | e raised to the power of a (e^a)", OperatorAction::Unary(f64::exp)),
    "ceil" => ("Rounding", "a ceil | Ceiling (rounds up)", OperatorAction::Unary(f64::ceil)),
    "floor" => ("Rounding", "a floor | Floor (rounds down)", OperatorAction::Unary(f64::floor)),
    "deg" => ("Conversions", "a deg | Convert angle from radians to degrees", OperatorAction::Unary(rad_to_deg)),
    "rad" => ("Conversions", "a rad | Convert angle from degrees to radians", OperatorAction::Unary(deg_to_rad)),

    // Special/Custom Logic (Handled explicitly in process_token's Special match)
    "!" => ("Combinatorics", "n ! | Factorial (n!)", OperatorAction::Special("factorial")),
    "P" => ("Combinatorics", "n k P | Permutations P(n, k)", OperatorAction::Special("permutations")),
    "C" => ("Combinatorics", "n k C | Combinations C(n, k)", OperatorAction::Special("combinations")),
    "<>" => ("Stack", "a b <> | Swap the top two items", OperatorAction::Special("swap")),
    "c" => ("Stack", "c | Clear the stack", OperatorAction::Special("clear")),
    "a" => ("Stack", "a | Recall last successful answer", OperatorAction::Special("answer")),
    "sto" => ("Memory", "value \"key\" sto | Store value to key", OperatorAction::Special("store")),
    "rcl" => ("Memory", "\"key\" rcl | Recall value from key", OperatorAction::Special("recall")),
    "hex" => ("Display", "a hex | Display a in hexadecimal (i64 cast)", OperatorAction::Special("display_base")),
    "bin" => ("Display", "a bin | Display a in binary (i64 cast)", OperatorAction::Special("display_base")),
    "oct" => ("Display", "a oct | Display a in octal (i64 cast)", OperatorAction::Special("display_base")),
    "help" => ("Meta", "help [func] | List all functions or show usage for [func]", OperatorAction::Special("help")),
};

/// Represents an item that can be placed on the RPN stack.
/// It can be a floating-point number or a string key for storage.
#[derive(Debug, Clone)]
enum StackItem {
    Number(f64),
    Key(String),
}

/// Displays help for all functions or a specific function, reading from the centralized map.
fn display_help(token: &str) -> Result<(), &'static str> {
    if token.is_empty() {
        // List all available functions, grouped by type
        println!("\n--- Available Functions ---");

        // Use a standard HashMap for runtime grouping
        let mut grouped_help: HashMap<&'static str, Vec<(&'static str, &'static str)>> =
            HashMap::new();
        // Iterate over the centralized map to extract help data only
        for (func, (group, usage, _action)) in OPERATOR_DATA.entries() {
            grouped_help.entry(group).or_default().push((*func, *usage));
        }

        let groups = vec![
            "Binary", // Combined Arithmetic and Log/Atan2
            "Unary",
            "Rounding",
            "Conversions",
            "Combinatorics",
            "Constants",
            "Stack",
            "Memory",
            "Display",
            "Meta",
        ];

        for group in groups {
            if let Some(items) = grouped_help.get(group) {
                println!("\n  ✨ {}:", group);
                for (func, usage) in items {
                    println!("    - {:<5} | {}", func, usage);
                }
            }
        }
    } else {
        // Show help for a specific function
        if let Some((group, usage, _action)) = OPERATOR_DATA.get(token) {
            println!("\n--- Help for '{}' ---", token);
            println!("  Type: {}", group);
            println!("  Usage: {}", usage);
        } else {
            return Err("Function not found. Type 'help' for a full list.");
        }
    }

    Ok(())
}

/// Calculates permutations P(n, k) = n! / (n - k)!.
/// Pops two numbers (k, n), calculates P(n, k), and pushes the result.
fn permutations(stack: &mut Vec<StackItem>) -> Result<(), &'static str> {
    // RPN: requires n then k, so pop k first, then n.
    // Use an error handler helper to avoid repeating pop/push logic on failure
    let handle_error = |stack: &mut Vec<StackItem>, n_val, k_val, err| {
        // Push back in reverse order (k then n) since they are currently popped
        stack.push(StackItem::Number(n_val));
        stack.push(StackItem::Number(k_val));
        Err(err)
    };

    let k_val = match stack.pop() {
        Some(StackItem::Number(val)) => val,
        _ => return Err("P(n, k) requires two numbers (n, k) on the stack (missing k)"),
    };
    let n_val = match stack.pop() {
        Some(StackItem::Number(val)) => val,
        _ => {
            return handle_error(
                stack,
                0.0,
                k_val,
                "P(n, k) requires two numbers (n, k) on the stack (missing n)",
            );
        }
    };

    // Check bounds and non-negativity
    if n_val < 0.0 || k_val < 0.0 {
        return handle_error(stack, n_val, k_val, "P(n, k) requires non-negative inputs.");
    }

    let n = n_val.round() as i64;
    let k = k_val.round() as i64;

    if k > n {
        return handle_error(
            stack,
            n_val,
            k_val,
            "P(n, k): n must be greater than or equal to k.",
        );
    }

    // Check for large input to avoid overflow in intermediate calculation (max 20!)
    if n > 20 || k > 20 {
        return handle_error(
            stack,
            n_val,
            k_val,
            "P(n, k): Inputs too large; max n is 20.",
        );
    }

    // P(n, k) = n * (n-1) * ... * (n-k+1)
    let result = (n - k + 1..=n).map(|i| i as f64).product();

    stack.push(StackItem::Number(result));
    Ok(())
}

/// Calculates combinations C(n, k) = n! / (k! * (n - k)!).
/// Pops two numbers (k, n), calculates C(n, k), and pushes the result.
fn combinations(stack: &mut Vec<StackItem>) -> Result<(), &'static str> {
    // RPN: requires n then k, so pop k first, then n.
    let handle_error = |stack: &mut Vec<StackItem>, n_val, k_val, err| {
        // Push back in reverse order (k then n) since they are currently popped
        stack.push(StackItem::Number(n_val));
        stack.push(StackItem::Number(k_val));
        Err(err)
    };

    let k_val = match stack.pop() {
        Some(StackItem::Number(val)) => val,
        _ => return Err("C(n, k) requires two numbers (n, k) on the stack (missing k)"),
    };
    let n_val = match stack.pop() {
        Some(StackItem::Number(val)) => val,
        _ => {
            return handle_error(
                stack,
                0.0,
                k_val,
                "C(n, k) requires two numbers (n, k) on the stack (missing n)",
            );
        }
    };

    // Check bounds and non-negativity
    if n_val < 0.0 || k_val < 0.0 {
        return handle_error(stack, n_val, k_val, "C(n, k) requires non-negative inputs.");
    }

    let n = n_val.round() as i64;
    let k = k_val.round() as i64;

    if k > n {
        return handle_error(
            stack,
            n_val,
            k_val,
            "C(n, k): n must be greater than or equal to k.",
        );
    }

    // Check for large input (C(n, k) can exceed f64, e.g., C(67, 33))
    // A safe upper limit for n, considering the final f64 result is ~10^308
    if n > 170 {
        return handle_error(
            stack,
            n_val,
            k_val,
            "C(n, k): n is too large (> 170) for f64 result.",
        );
    }

    // Optimization: C(n, k) = C(n, n-k)
    let k_eff = std::cmp::min(k, n - k);

    // C(n, k) = (n * (n-1) * ... * (n-k+1)) / k!
    let mut result = 1.0;
    for i in 0..k_eff {
        // Multiplies by (n-i) and divides by (i+1) in the same loop for better precision
        result = result * (n as f64 - i as f64) / (i as f64 + 1.0);
    }

    stack.push(StackItem::Number(result));
    Ok(())
}

/// Helper to convert various Unicode digits and separators to ASCII digits and standard separators.
fn unicode_to_ascii(c: char) -> char {
    match c {
        // Persian (Extended Arabic-Indic) Digits
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

        // Standard Arabic (Arabic-Indic) Digits
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

        // Persian/Arabic Separators
        '٫' => '.', // Arabic Decimal Separator -> ASCII Period
        '٬' => ',', // Arabic Thousands Separator -> ASCII Comma

        _ => c, // Leave all other characters unchanged
    }
}

/// Applies an operation to the top f64 value on the stack, modifying it in place.
fn unary_calculate(
    stack: &mut Vec<StackItem>,
    operation: UnaryHandler,
) -> Result<(), &'static str> {
    let val = match stack.last_mut() {
        Some(StackItem::Number(val)) => val,
        _ => return Err("Unary operator requires one number on the stack"),
    };

    // Read the value, perform the operation, and write back to the reference
    *val = operation(*val);

    Ok(())
}

/// Binary function for two-operand operations (e.g., +, -, *, /).
/// Pops two numbers (a and b), applies the function (a op b), and pushes the result.
fn calculate(
    stack: &mut Vec<StackItem>,
    op: BinaryHandler,
    _op_symbol: &str,
) -> Result<(), &'static str> {
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
            // Push the second operand back before erroring
            stack.push(StackItem::Number(b));
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

/// Calculates the factorial of n (n!).
/// Returns an error if n is negative, non-integer, or too large.
fn factorial(stack: &mut Vec<StackItem>) -> Result<(), &'static str> {
    // 1. Pop the number
    let val = match stack.pop() {
        Some(StackItem::Number(val)) => val,
        _ => return Err("Factorial '!' requires one number on the stack"),
    };

    // 2. Check for negative input
    if val < 0.0 {
        stack.push(StackItem::Number(val));
        return Err("Factorial '!' requires a non-negative number.");
    }

    // 3. Check for large input (21! is already too large for f64)
    if val > 20.0 {
        stack.push(StackItem::Number(val));
        return Err("Factorial '!' is too large; max supported value is 20.");
    }

    // 4. Round to the nearest integer and calculate
    let n_int = val.round() as u64;
    let result = (1..=n_int).map(|i| i as f64).product();

    // 5. Push result
    stack.push(StackItem::Number(result));
    Ok(())
}

/// Reads the last f64, casts it to i64, prints it in the given base.
/// The stack is NOT modified.
fn display_base(stack: &mut Vec<StackItem>, token: &str) -> Result<(), &'static str> {
    // 1. Check stack and get number (read-only access)
    let a = match stack.last() {
        Some(StackItem::Number(val)) => *val,
        _ => return Err("Base conversion requires one number on the stack"),
    };

    // 2. Cast to integer (truncates fractional part)
    let int_val = a as i64;
    let (prefix, base_str) = match token {
        "hex" => ("0x", format!("{:X}", int_val)),
        "oct" => ("0o", format!("{:o}", int_val)),
        "bin" => ("0b", format!("{:b}", int_val)),
        _ => return Err("Invalid base token"),
    };

    // 3. Print the result outside the stack
    println!("\n{} Base: {}{}", token, prefix, base_str);

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
    let cleaned_token: String = token
        .chars()
        .map(unicode_to_ascii)
        .filter(|c| *c != ',')
        .collect();

    if let Ok(num) = cleaned_token.parse::<f64>() {
        stack.push(StackItem::Number(num));
        return Ok(());
    }

    // 3. Handle Commands and Operators via Centralized Map Lookup
    if let Some((_group, _usage, action)) = OPERATOR_DATA.get(token) {
        match action {
            OperatorAction::PushConstant(val) => {
                stack.push(StackItem::Number(*val));
                Ok(())
            }
            OperatorAction::Unary(handler) => unary_calculate(stack, *handler),
            OperatorAction::Binary(handler) => calculate(stack, *handler, token),
            OperatorAction::Special(name) => {
                // Execute special commands which need custom state access
                match *name {
                    "factorial" => factorial(stack),
                    "permutations" => permutations(stack),
                    "combinations" => combinations(stack),
                    "swap" => swap(stack),
                    "clear" => {
                        stack.clear();
                        Ok(())
                    }
                    "answer" => {
                        if let Some(val) = *last_answer {
                            stack.push(StackItem::Number(val));
                            Ok(())
                        } else {
                            Err("No previous answer available ('a' is empty)")
                        }
                    }
                    // Store (sto)
                    "store" => {
                        let key = match stack.pop() {
                            Some(StackItem::Key(k)) => k,
                            _ => {
                                return Err(
                                    "STO requires a string key (e.g., \"rate\") as the last item",
                                );
                            }
                        };
                        let val = match stack.pop() {
                            Some(StackItem::Number(v)) => v,
                            _ => return Err("STO requires a number value before the key"),
                        };
                        storage.insert(key, val);
                        Ok(())
                    }
                    // Recall (rcl)
                    "recall" => {
                        let key = match stack.pop() {
                            Some(StackItem::Key(k)) => k,
                            _ => {
                                return Err(
                                    "RCL requires a string key (e.g., \"rate\") as the last item",
                                );
                            }
                        };
                        if let Some(&val) = storage.get(&key) {
                            stack.push(StackItem::Number(val));
                            Ok(())
                        } else {
                            Err("Storage key not found")
                        }
                    }
                    "display_base" => display_base(stack, token),
                    "help" => {
                        // Custom RPN help logic
                        let target_item = stack.pop();
                        match target_item {
                            Some(StackItem::Key(key)) => {
                                let func_name = key.trim_matches('"').to_lowercase();
                                if OPERATOR_DATA.contains_key(func_name.as_str()) {
                                    display_help(func_name.as_str())
                                } else {
                                    stack.push(StackItem::Key(key));
                                    display_help("")
                                }
                            }
                            Some(StackItem::Number(val)) => {
                                stack.push(StackItem::Number(val));
                                display_help("")
                            }
                            None => display_help(""),
                        }
                    }
                    _ => Err("Internal operator error (Special command missing handler)"),
                }
            }
        }
    } else {
        Err("Unrecognized token or operator")
    }
}

fn main() {
    let mut stack: Vec<StackItem> = Vec::new();
    let mut last_answer: Option<f64> = None;
    let mut storage: HashMap<String, f64> = HashMap::new();

    println!("Welcome to kalk-rs (RPN Calculator). Type 'exit' to quit.");
    println!("Type 'help' for a list of all functions or '\"func\" help' for specific usage.");

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
}
