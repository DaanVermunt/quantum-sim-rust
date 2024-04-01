pub fn f64_equal(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.000000001
}