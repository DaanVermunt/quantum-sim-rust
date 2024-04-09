use std::collections::HashMap;

use rand::Rng;

use crate::{
    c,
    matrix::{complex::C, matrix::Matrix},
    util::{binary_string_to_int, index_to_binary_string, mod_power},
};

fn pick_a(n: u32) -> u32 {
    // Pick random number a < n
    let mut rng = rand::thread_rng();
    rng.gen_range(2..n)
}

fn gcd<T: std::ops::Rem<Output = T> + Copy + PartialEq + Default>(a: T, b: T) -> T {
    if b == T::default() {
        return a;
    }
    gcd(b, a % b)
}

fn lcm<
    T: std::ops::Div<Output = T>
        + std::ops::Rem<Output = T>
        + std::ops::Mul<Output = T>
        + Copy
        + PartialEq
        + Default,
>(
    a: T,
    b: T,
) -> T {
    a * b / gcd(a, b)
}

fn lcm_vec<
    T: std::ops::Div<Output = T>
        + std::ops::Rem<Output = T>
        + std::ops::Mul<Output = T>
        + Copy
        + PartialEq
        + Default,
>(
    nums: Vec<T>,
) -> T {
    let mut res = nums[0];
    for i in 1..nums.len() {
        if nums[i] == T::default() {
            continue;
        }
        res = lcm(res, nums[i]);
    }
    res
}

fn period_in_ints(nbmrs: Vec<usize>) -> usize {
    let mut min = 10e5 as usize;

    for i in nbmrs.iter() {
        if i.clone() < min {
            min = i.clone();
        }
    }

    let mut subtrcts: Vec<usize> = vec![];

    for i in nbmrs.iter() {
        if i.clone() == min {
            continue;
        }

        subtrcts.push(i.clone() - min);
    }

    let mut attempt = gcd(subtrcts[0], subtrcts[1]);

    for i in 2..subtrcts.len() {
        attempt = gcd(attempt, subtrcts[i])
    }

    attempt
}

fn in_fraction(x: f64) -> (usize, usize) {
    const EPSILON: f64 = 1e-9; // Adjust epsilon based on your precision requirement

    // Start with a denominator of 1
    let mut denominator = 1;

    // Calculate the floating-point representation of the input fraction
    let mut n = (x * denominator as f64).round() as usize;
    let mut frac = (n as f64) / (denominator as f64);

    // Increase the denominator until we find the closest fraction
    while (frac - x).abs() > EPSILON {
        denominator += 1;
        n = (x * denominator as f64).round() as usize;
        frac = (n as f64) / (denominator as f64);
    }

    // Return the simplified fraction
    let gcd = gcd(n, denominator);
    (n / gcd, denominator / gcd)
}

fn get_m_probability_dist(m: Matrix, n_bits: usize) -> Vec<(usize, C)> {
    if !m.is_vector() {
        panic!("M should be a vector");
    }

    let mut res: Vec<(usize, C)> = vec![];
    for i in 1..m.size().0 {
        let v = m.data[i][0];
        if v == c!(0) {
            continue;
        }

        let binary_string = index_to_binary_string(i, n_bits * 3);
        let m_string = binary_string[0..(n_bits * 2)].to_string();
        let m = binary_string_to_int(m_string);

        res.push((m, v))
    }

    res
}

fn get_n_probability_dist(m: Matrix, n_bits: usize) -> Vec<(usize, C)> {
    if !m.is_vector() {
        panic!("M should be a vector");
    }

    let mut res: Vec<(usize, C)> = vec![];
    for i in 1..m.size().0 {
        let v = m.data[i][0];
        if v == c!(0) {
            continue;
        }

        let binary_string = index_to_binary_string(i, n_bits * 3);
        let n_string = binary_string[(n_bits * 2)..(n_bits * 3)].to_string();
        let n = binary_string_to_int(n_string);

        res.push((n, v))
    }

    res
}

fn get_m(binary_string: String, n_bits: usize) -> usize {
    let m_string = binary_string[0..(n_bits * 2)].to_string();
    binary_string_to_int(m_string)
}

fn find_period(a: u32, n: u32) -> u32 {
    let n_bits = ((n + 1) as f64).log2().ceil() as u32;
    let m_bits = 2 * n_bits;

    let size = m_bits + n_bits;
    println!("Size: {} = m({}) + n({})", size, m_bits, n_bits);

    let mut script = format!("INITIALIZE R {}\n", size);

    script.push_str("U TENSOR G_H G_H\n");

    for _ in 0..(m_bits - 2) {
        script.push_str("U TENSOR U G_H\n");
    }

    let n_size = (2 as u32).clone().pow(n_bits.clone() as u32) as usize;
    script.push_str(format!("U TENSOR U G_I_{}\n", n_size).as_str());

    script.push_str("APPLY U R\n");
    script.push_str(format!("APPLY G_Uf_{}_{} R\n", a, n).as_str());
    script.push_str(format!("SELECT S R {} {}\n", m_bits, n_bits + m_bits).as_str());
    script.push_str("MEASURE S RES\n");

    // script.push_str(format!("U TENSOR G_QFTI_{} G_I_{}\n", m_bits, n_size).as_str());
    // script.push_str("MEASURE R RES_TMP\n"); // NB This should not work due to the quantum nature, but we leverage the fact that it is a sim here
    // script.push_str("APPLY U R\n");
    script.push_str("MEASURE R RES1\n"); // NB This should not work due to the quantum nature, but we leverage the fact that it is a sim here
    script.push_str("MEASURE R RES2\n");
    script.push_str("MEASURE R RES3\n");
    script.push_str("MEASURE R RES4\n");
    script.push_str("MEASURE R RES5\n");
    script.push_str("MEASURE R RES6\n");
    script.push_str("MEASURE R RES7\n");

    let res = crate::quantum_assembler::run(script);

    if res.is_err() {
        panic!(
            "Error running quantum assembler script {:?}",
            res.err().unwrap()
        );
    }

    let res = res.unwrap();

    let c1 = get_m((&res.get("RES1").unwrap().1).clone(), n_bits as usize);
    let c2 = get_m((&res.get("RES2").unwrap().1).clone(), n_bits as usize);
    let c3 = get_m((&res.get("RES3").unwrap().1).clone(), n_bits as usize);
    let c4 = get_m((&res.get("RES4").unwrap().1).clone(), n_bits as usize);
    let c5 = get_m((&res.get("RES5").unwrap().1).clone(), n_bits as usize);
    let c6 = get_m((&res.get("RES6").unwrap().1).clone(), n_bits as usize);
    let c7 = get_m((&res.get("RES7").unwrap().1).clone(), n_bits as usize);

    period_in_ints(vec![c1, c2, c3, c4, c5, c6, c7]) as u32
}

fn find_factors(r: u32, a: u32, n: u32) -> Option<(u32, u32)> {
    if r % 2 != 0 {
        return None;
    }

    if mod_power(a, r, n) == n - 1 {
        return None;
    }

    let g = gcd(mod_power(a, r / 2, n) + 1, n);

    if g == 1 || g == n {
        return None;
    }

    return Some((g, n / g));
}

pub fn shors(n: u32) -> Option<(u32, u32)> {
    // 0. Validate log2(n) < max_q_bits

    // 1. Use polynomial to determine if n is power of a prime or a prime, if so return
    // For now will skip and assume n is p * q with p and q both prime

    // 2. Pick random number a < n
    for i in 0..10 {
        let a = pick_a(n);

        // 2.1 if gcd(a, n) != 1, a is a the factor of n we were looking for
        if gcd(a, n) != 1 {
            return Some((gcd(a, n), n / gcd(a, n)));
        }

        // 3. Use quantum algorithm to find period r of a^x mod n
        let r = find_period(a, n);
        println!("a {}, for n {} => period {}", a, n, r);

        let res = find_factors(r, a, n);
        if res.is_none() {
            return None; // TODO: SHOULD CONTINUE THE LOOP
        }

        return res;
    }
    panic!("COULD NOT FIND A VALID R")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shors() {
        let n = 15;
        let (p, q) = shors(n).unwrap();
        assert_eq!(p * q, n);

        let n = 6;
        let (p, q) = shors(n).unwrap();
        assert_eq!(p * q, n);

        let n = 14;
        let (p, q) = shors(n).unwrap();
        assert_eq!(p * q, n);
    }

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(10, 15), 5);
        assert_eq!(gcd(10, 20), 10);
        assert_eq!(gcd(10, 21), 1);
        assert_eq!(gcd(21, 7), 7);
    }

    #[test]
    fn test_lcm() {
        assert_eq!(lcm_vec(vec![2, 3, 4]), 12);
        assert_eq!(lcm_vec(vec![2, 3, 5]), 30);
        assert_eq!(lcm_vec(vec![2, 3, 5, 15, 10]), 30);
        assert_eq!(lcm_vec(vec![21, 6]), 42);
    }

    #[test]
    fn test_in_fraction() {
        assert_eq!(in_fraction(0.25), (1, 4));
        assert_eq!(in_fraction(0.375), (3, 8));
        assert_eq!(in_fraction(0.6), (3, 5));
        assert_eq!(in_fraction(0.5), (1, 2));
    }

    #[test]
    fn test_find_period_in_int() {
        assert_eq!(period_in_ints(vec![2, 254, 14, 18]), 4);
        assert_eq!(period_in_ints(vec![2, 254, 14, 16]), 2);
        assert_eq!(period_in_ints(vec![7, 13, 19, 28]), 3);
        assert_eq!(period_in_ints(vec![10, 20, 1005]), 5);
    }

    #[test]
    fn test_find_period() {
        // assert_eq!(find_period(2, 23), 7);
        assert_eq!(find_period(2, 15), 4);
        // assert_eq!(find_period(6, 371), 26);
        // assert_eq!(find_period(24, 371), 78);
    }

    #[test]
    fn test_find_factors() {
        let r = 4;
        let a = 2;
        let n = 15;
        let res = find_factors(r, a, n);
        assert_eq!(res, Some((5, 3)));

        let r = 3;
        let res = find_factors(r, a, n);
        assert_eq!(res, None);

        let r = 26;
        let a = 6;
        let n = 371;

        let res = find_factors(r, a, n);
        assert_eq!(res, None);

        assert_eq!(find_factors(78, 24, 371), Some((7, 53)));
    }
}
