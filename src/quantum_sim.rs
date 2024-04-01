use rand::{thread_rng, Rng};

use crate::{c, complex::*, mat, matrix::*, util::f64_equal};

pub fn prob_at(m: &Matrix, idx: usize) -> f64 {
    if (idx >= m.data.len()) || (m.data[0].len() != 1) {
        panic!("Invalid index");
    }

    let norm = m.norm();
    let val = m.data[idx][0].modulus();

    val.powf(2.0) / norm.powf(2.0)
}

fn index_to_binary_string(index: usize, n: usize) -> String {
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

pub fn measure_vec(m: &Matrix) -> String {
    let qbit_len = (m.size().0 as f64).log2().round() as usize;

    if !m.is_vector() || !f64_equal(qbit_len as f64, (m.size().0 as f64).log2()) {
        panic!("Invalid input for MEASURE, should be a vector of size power of two");
    }

    let mut rng = thread_rng();
    let val: f64 = rng.gen();

    let mut sum = 0.0;

    let mut pick = 0;
    for i in 0..m.size().0 {
        sum += prob_at(m, i);

        if val < sum {
            pick = i;
            break;
        }
    }

    return index_to_binary_string(pick, qbit_len);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measure_vec() {
        let m = mat![c!(0.0); c!(0.0); c!(0.0); c!(1.0);];
        let res = super::measure_vec(&m);
        assert_eq!(res, "11");
    }

    #[test]
    #[should_panic]
    fn test_measure_vec_panic() {
        let m = mat![c!(0.0); c!(0.0); c!(0.0); c!(1.0); c!(1.0);];
        let _ = super::measure_vec(&m);
    }

    #[test]
    fn test_measure_prob() {
        let m = mat![c!(0.0); c!(0.0); c!(0.7); c!(0.5)];
        let res = super::measure_vec(&m);

        assert!(res == "10" || res == "11");
    }
}
