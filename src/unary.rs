use super::{StackItem, UnaryHandler};
use std::f64::consts;
/// Applies an operation to the top f64 value on the stack, modifying it in place.
pub fn calculate(stack: &mut Vec<StackItem>, operation: UnaryHandler) -> Result<(), &'static str> {
    let val = match stack.last_mut() {
        Some(StackItem::Number(val)) => val,
        _ => return Err("Unary operator requires one number on the stack"),
    };

    // Read the value, perform the operation, and write back to the reference
    *val = operation(*val);

    Ok(())
}

pub fn rad_to_deg(rad: f64) -> f64 {
    rad * 180.0 / consts::PI
}
pub fn deg_to_rad(deg: f64) -> f64 {
    deg * consts::PI / 180.0
}
