use std::option;

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

pub fn qbit_length(m: &Matrix) -> usize {
    let qbit_len = (m.size().0 as f64).log2().round() as usize;

    if !m.is_vector() || !f64_equal(qbit_len as f64, (m.size().0 as f64).log2()) {
        panic!("Invalid input for MEASURE, should be a vector of size power of two");
    }

    qbit_len
}

pub fn measure_vec(m: &Matrix) -> String {
    let qbit_len = qbit_length(m);
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

pub fn measure_partial_vec(m: &Matrix, from: i32, to: i32) -> Matrix {
    assert!(m.is_vector(), "Invalid input measure, should be a vector");

    // GENERATE OPTIONS
    let size = (to - from) as usize;
    let two = 2 as usize;
    let option_vector_size = two.pow(size as u32) as usize;
    let mut options = Matrix::zero(option_vector_size, 1);
    let mut res_matrix = m.clone();
    let qbit_len = qbit_length(m);

    // GET PROBABILITIES FOR OPTIONS
    for i in 0..m.size().0 {
        let qbinary = index_to_binary_string(i, qbit_len);
        println!("Qbinary: {:?}", qbinary);
        for j in 0..option_vector_size {
            let qbinary_selection = index_to_binary_string(j, size);
            if qbinary[from as usize..to as usize] == qbinary_selection {
                options.data[j][0] = m.data[i][0] + options.data[j][0];
            }
        }
    }

    print!("Options: {:?}", options);

    // COLLAPSE STATE
    let res = measure_vec(&options);
    println!("Res {:?}", res);

    // UPDATE ORIGINAL STATE
    for i in 0..m.size().0 {
        let qbinary = index_to_binary_string(i, qbit_len);
        if qbinary[from as usize..to as usize] != res {
            res_matrix.data[i][0] = c!(0.0);
        }
    }

    res_matrix
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

    #[test]
    fn test_partial_measure() {
        let m = mat![c!(0.0); c!(1.0); c!(0.7); c!(0.5)];
        let res = super::measure_partial_vec(&m, 1, 2);

        assert!(
            res.clone() == mat![c!(0.0); c!(0.0); c!(0.7); c!(0.0)]
                || res.clone() == mat![c!(0.0); c!(1.0); c!(0.0); c!(0.5)]
        );

        let m = mat![c!(1.0); c!(1.0); c!(1.0); c!(1.0)];
        let res = super::measure_partial_vec(&m, 0, 2);
        assert_eq!(res.norm(), 1.0);
    }
}
