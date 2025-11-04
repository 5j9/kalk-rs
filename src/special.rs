use super::{OPERATOR_DATA, StackItem};
use std::collections::HashMap;

pub fn handle_special_operator(
    stack: &mut Vec<StackItem>,
    token: &str,
    special_name: &'static str,
    last_answer: &mut Option<f64>,
    storage: &mut HashMap<String, f64>,
) -> Result<(), &'static str> {
    match special_name {
        "factorial" => crate::special::factorial(stack),
        "permutations" => crate::special::permutations(stack),
        "combinations" => crate::special::combinations(stack),
        "swap" => crate::special::swap(stack),
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
        "store" => crate::special::store(stack, storage),
        "recall" => crate::special::recall(stack, storage),
        "display_base" => display_base(stack, token),
        "help" => {
            // Custom RPN help logic
            let target_item = stack.pop();
            match target_item {
                Some(StackItem::Key(key)) => {
                    let func_name = key.trim_matches('"').to_lowercase();
                    if crate::OPERATOR_DATA.contains_key(func_name.as_str()) {
                        display_help(func_name.as_str())
                    } else {
                        // Put the key back if it wasn't a function name
                        stack.push(StackItem::Key(key));
                        display_help("") // Show general help
                    }
                }
                Some(StackItem::Number(val)) => {
                    // Put the number back as it's not a function name
                    stack.push(StackItem::Number(val));
                    display_help("") // Show general help
                }
                None => display_help(""), // Show general help
            }
        }
        _ => Err("Internal operator error (Special command missing handler)"),
    }
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

/// Stores a number value from the stack into storage, identified by a key from the stack.
///
/// Expected stack order: [..., value, key]
/// Mutates the stack by popping two items. Mutates storage by inserting the key-value pair.
pub fn store(
    stack: &mut Vec<StackItem>,
    storage: &mut HashMap<String, f64>,
) -> Result<(), &'static str> {
    // 1. Pop the key (must be a Key variant)
    let key = match stack.pop() {
        Some(StackItem::Key(k)) => k.to_string(), // Convert Box<str> to String for HashMap key
        // If the item wasn't a Key, return an error.
        item => {
            // If we popped something but it was the wrong type, push it back.
            if let Some(i) = item {
                stack.push(i);
            }
            return Err("STO requires a string key (e.g., \"rate\") as the last item");
        }
    };

    // 2. Pop the value (must be a Number variant)
    let val = match stack.pop() {
        Some(StackItem::Number(v)) => v,
        _ => {
            // Crucial: The key was already popped. If value pop fails,
            // we have an inconsistent state. A simple error is sufficient here.
            return Err("STO requires a number value before the key");
        }
    };

    storage.insert(key, val);
    Ok(())
}

/// Recalls a number value from storage onto the stack, identified by a key from the stack.
///
/// Expected stack order: [..., key]
/// Mutates the stack by popping the key and pushing the retrieved number.
pub fn recall(
    stack: &mut Vec<StackItem>,
    storage: &HashMap<String, f64>,
) -> Result<(), &'static str> {
    // 1. Pop the key
    let key = match stack.pop() {
        Some(StackItem::Key(k)) => k.to_string(),
        item => {
            if let Some(i) = item {
                stack.push(i);
            }
            return Err("RCL requires a string key (e.g., \"rate\") as the last item");
        }
    };

    // 2. Look up the value and push it back
    if let Some(&val) = storage.get(&key) {
        stack.push(StackItem::Number(val));
        Ok(())
    } else {
        // If the key wasn't found, push the key back onto the stack
        // (as the user might try a different key)
        stack.push(StackItem::Key(key.into()));
        Err("Storage key not found")
    }
}

/// Calculates the factorial of n (n!).
/// Returns an error if n is negative, non-integer, or too large.
pub fn factorial(stack: &mut Vec<StackItem>) -> Result<(), &'static str> {
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

/// Calculates permutations P(n, k) = n! / (n - k)!.
/// Pops two numbers (k, n), calculates P(n, k), and pushes the result.
pub fn permutations(stack: &mut Vec<StackItem>) -> Result<(), &'static str> {
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
pub fn combinations(stack: &mut Vec<StackItem>) -> Result<(), &'static str> {
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

/// Swaps the position of the last two number values on the stack.
pub fn swap(stack: &mut Vec<StackItem>) -> Result<(), &'static str> {
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
