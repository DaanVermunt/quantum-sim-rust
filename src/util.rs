pub fn f64_equal(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.000000001
}

pub fn min_bit_size(n: u32) -> u32 {
    ((n + 1) as f64).log2().ceil() as u32
}

pub fn mod_power(a: u32, x: u32, n: u32) -> u32 {
    let mut res = 1;
    for _ in 0..x {
        res = (res * a) % n;
    }
    res
}

pub fn binary_string_to_int(s: String) -> usize {
    let mut result = 0;
    for c in s.chars() {
        result <<= 1;
        if c == '1' {
            result |= 1;
        }
    }
    result
}

pub fn index_to_binary_string(index: usize, n: usize) -> String {
    let mut result = String::with_capacity(n);
    for i in (0..n).rev() {
        if index & (1 << i) != 0 {
            result.push('1');
        } else {
            result.push('0');
        }
    }
    result
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_size() {
        assert_eq!(min_bit_size(1), 1);
        assert_eq!(min_bit_size(15), 4);
        assert_eq!(min_bit_size(2), 2);
        assert_eq!(min_bit_size(100), 7);
    }

    #[test]
    fn test_binary_to_int() {
        assert_eq!(binary_string_to_int("101".to_string()), 5);
        assert_eq!(binary_string_to_int("10101".to_string()), 21);
        assert_eq!(binary_string_to_int("00000".to_string()), 0);
        assert_eq!(binary_string_to_int("0001".to_string()), 1);
    }
}
