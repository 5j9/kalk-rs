use super::{BinaryHandler, StackItem};

pub fn log_op(a: f64, b: f64) -> f64 {
    a.log(b)
}
pub fn percent_change(a: f64, b: f64) -> f64 {
    (b - a) / a * 100.0
}
pub fn power_op(a: f64, b: f64) -> f64 {
    a.powf(b)
}
pub fn atan2_op(y: f64, x: f64) -> f64 {
    y.atan2(x)
}

/// Binary function for two-operand operations (e.g., +, -, *, /).
/// Pops two numbers (a and b), applies the function (a op b), and pushes the result.
pub fn calculate(
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
