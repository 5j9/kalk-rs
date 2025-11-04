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
