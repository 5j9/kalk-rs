use std::io::{self, Write};

fn main() {
    // 1. The RPN stack to hold floating-point numbers
    let mut stack: Vec<f64> = Vec::new();

    println!("Welcome to kalk-rs (RPN Calculator). Type 'exit' to quit.");

    // 2. Main input loop
    loop {
        // Display the current stack state
        print!("Stack: {:?}\n> ", stack);
        // Flush the output buffer so the prompt appears immediately
        io::stdout().flush().unwrap();

        // Read user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        // 3. Process tokens
        let input = input.trim();

        if input == "exit" {
            break;
        }

        // Split input into tokens (numbers and operators)
        for token in input.split_whitespace() {
            if let Err(e) = process_token(&mut stack, token) {
                eprintln!("Error: {}", e);
                // Clear stack or handle error as needed
                stack.clear();
                break;
            }
        }
    }
}

// Add this function right below the main function in src/main.rs
fn process_token(stack: &mut Vec<f64>, token: &str) -> Result<(), &'static str> {
    // 1. Convert Persian digits to Arabic and remove commas
    let cleaned_token: String = token
        .chars()
        .map(persian_to_arabic) // Convert all Persian digits
        .filter(|c| *c != ',') // Remove thousand separators
        .collect();

    // 2. Try to parse the cleaned token as a number
    if let Ok(num) = cleaned_token.parse::<f64>() {
        stack.push(num);
        return Ok(());
    }

    // Handle arithmetic operations
    match token {
        "+" => calculate(stack, |a, b| a + b, "+"),
        "-" => calculate(stack, |a, b| a - b, "-"),
        "*" | "x" => calculate(stack, |a, b| a * b, "*"),
        "/" => calculate(stack, |a, b| a / b, "/"),
        "<>" => swap(stack),
        "c" => {
            stack.clear();
            Ok(())
        }
        "pi" => {
            stack.push(std::f64::consts::PI); // Pushes the value of Pi
            Ok(())
        }
        "e" => {
            stack.push(std::f64::consts::E); // Pushes the value of Euler's number
            Ok(())
        }
        "**" => calculate(stack, |a, b| a.powf(b), "**"),
        "sqrt" => unary_calculate(stack, f64::sqrt),
        _ => Err("Unrecognized token or operator"),
    }
}

// Add this function below process_token in src/main.rs
fn calculate<F>(stack: &mut Vec<f64>, op: F, _op_symbol: &str) -> Result<(), &'static str>
where
    F: Fn(f64, f64) -> f64,
{
    // RPN needs two operands: pop the second-to-last (b) and last (a)
    let b = stack.pop().ok_or("Not enough values for operation")?;
    let a = stack.pop().ok_or("Not enough values for operation")?;

    // Perform the calculation and push the result
    stack.push(op(a, b));
    Ok(())
}

// Add this new function below `calculate` in src/main.rs
fn swap(stack: &mut Vec<f64>) -> Result<(), &'static str> {
    if stack.len() < 2 {
        return Err("Not enough values on the stack to swap");
    }

    // Get the index of the second-to-last item (length - 2)
    let idx_b = stack.len() - 2;

    // Get the index of the last item (length - 1)
    let idx_a = stack.len() - 1;

    // Use a built-in method to safely swap the elements
    stack.swap(idx_a, idx_b);

    Ok(())
}

// Unary function for single-operand operations (e.g., sqrt, sin, log)
fn unary_calculate<F>(stack: &mut Vec<f64>, op: F) -> Result<(), &'static str>
where
    F: Fn(f64) -> f64,
{
    // RPN needs one operand: pop the last (a)
    let a = stack.pop().ok_or("Not enough values for unary operation")?;

    // Perform the calculation and push the result
    stack.push(op(a));
    Ok(())
}

// Add this helper function to src/main.rs
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
        // Leave all other characters (like '.', ',', '+', etc.) unchanged
        _ => c,
    }
}
