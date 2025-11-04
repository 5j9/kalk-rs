use phf::{Map, phf_map};
use std::collections::HashMap;
use std::f64::consts;
use std::io::{self, Write};
use thousands::Separable;

/// Type alias for a function that operates on one f64 and returns an f64.
type UnaryHandler = fn(f64) -> f64;
/// Type alias for a function that operates on two f64s and returns an f64.
type BinaryHandler = fn(f64, f64) -> f64;
mod binary;
mod special;
mod unary;

/// Represents an item that can be placed on the RPN stack.
/// It can be a floating-point number or a string key for storage.
#[derive(Debug, Clone)]
pub enum StackItem {
    Number(f64),
    Key(String),
}

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
    "**" => ("Binary", "a b ** | Power (a^b)", OperatorAction::Binary(binary::power_op)),
    "%" => ("Binary", "a b % | Euclidean Remainder (a mod b)", OperatorAction::Binary(f64::rem_euclid)),
    "%%" => ("Binary", "a b %% | Percent Change ((b - a) / a * 100)", OperatorAction::Binary(binary::percent_change)),
    "log" => ("Binary", "a b log | Logarithm (log_b(a))", OperatorAction::Binary(binary::log_op)),
    "atan2" => ("Binary", "y x atan2 | Arc tangent of y/x (result in radians)", OperatorAction::Binary(binary::atan2_op)),

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
    "deg" => ("Conversions", "a deg | Convert angle from radians to degrees", OperatorAction::Unary(unary::rad_to_deg)),
    "rad" => ("Conversions", "a rad | Convert angle from degrees to radians", OperatorAction::Unary(unary::deg_to_rad)),

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
    "help" => ("Meta", "\"func_name\" help | List all functions or show usage for [func_name]", OperatorAction::Special("help")),
};

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

/// The core function to process a single input token.
pub fn process_token(
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
            OperatorAction::Unary(handler) => unary::calculate(stack, *handler),
            OperatorAction::Binary(handler) => binary::calculate(stack, *handler, token),
            OperatorAction::Special(name) => {
                special::handle_special_operator(stack, token, *name, last_answer, storage)
            }
        }
    } else {
        Err("Unrecognized token or operator")
    }
}

pub fn main_app_loop() {
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
