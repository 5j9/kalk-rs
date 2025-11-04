use std::collections::HashMap;

// We need to bring the parent's StackItem into scope to use it.
// The code in src/lib.rs or src/main.rs acts as the 'super' module.
use super::StackItem;

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
