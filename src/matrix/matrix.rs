use std::ops::{Add, Mul};

use crate::{
    c,
    util::{min_bit_size, mod_power},
};

use super::complex::C;

#[derive(Debug, Clone, PartialEq)]
pub struct Matrix {
    pub data: Vec<Vec<C>>,
}

impl Add for Matrix {
    type Output = Matrix;

    fn add(self, other: Matrix) -> Matrix {
        assert_eq!(self.data.len(), other.data.len());
        assert_eq!(self.data[0].len(), other.data[0].len());

        let mut data = vec![vec![c!(0); self.data.len()]; self.data[0].len()];
        for i in 0..self.data.len() {
            for j in 0..self.data[0].len() {
                data[i][j] = self.data[i][j] + other.data[i][j];
            }
        }
        Matrix { data: data }
    }
}

impl Mul for &Matrix {
    type Output = Matrix;

    fn mul(self, other: &Matrix) -> Matrix {
        self.multiply(&other)
    }
}

impl Mul for &mut Matrix {
    type Output = Matrix;

    fn mul(self, other: &mut Matrix) -> Matrix {
        self.multiply(&other)
    }
}

impl Mul for Matrix {
    type Output = Matrix;

    fn mul(self, other: Matrix) -> Matrix {
        self.multiply(&other)
    }
}

impl Matrix {
    pub fn new<T: Into<Vec<Vec<C>>>>(data: T) -> Matrix {
        Matrix { data: data.into() }
    }

    pub fn zero_sq(size: usize) -> Matrix {
        Matrix::zero(size, size)
    }

    pub fn zero(rows: usize, cols: usize) -> Matrix {
        let data = vec![vec![c!(0); cols]; rows];
        Matrix { data }
    }

    pub fn set(&self, row: usize, col: usize, value: C) -> Matrix {
        let mut data = self.data.clone();
        data[row][col] = value;
        Matrix { data }
    }

    pub fn identity(size: usize) -> Matrix {
        let mut data = vec![vec![c!(0); size]; size];
        for i in 0..size {
            data[i][i] = c!(1);
        }
        Matrix { data }
    }

    pub fn transpose(&self) -> Matrix {
        let mut data = vec![vec![c!(0); self.data.len()]; self.data[0].len()];
        for i in 0..self.data.len() {
            for j in 0..self.data[0].len() {
                data[j][i] = self.data[i][j];
            }
        }
        Matrix { data }
    }

    pub fn conjugate(&self) -> Matrix {
        let mut data = self.data.clone();
        for i in 0..self.data.len() {
            for j in 0..self.data[0].len() {
                data[i][j] = self.data[i][j].conjugate();
            }
        }
        Matrix { data }
    }

    pub fn adjoint(&self) -> Matrix {
        self.conjugate().transpose()
    }

    pub fn normalized(&self) -> Matrix {
        let norm = self.norm();
        self.scalar_mul(c!(1.0 / norm))
    }

    pub fn negative_inverse(&self) -> Matrix {
        let mut data = self.data.clone();
        for i in 0..self.data.len() {
            for j in 0..self.data[0].len() {
                data[i][j] = c!(-1) * self.data[i][j];
            }
        }
        Matrix { data }
    }

    pub fn scalar_mul(&self, scalar: C) -> Matrix {
        let mut data = self.data.clone();
        for i in 0..self.data.len() {
            for j in 0..self.data[0].len() {
                data[i][j] = self.data[i][j] * scalar;
            }
        }
        Matrix { data }
    }

    pub fn multiply(&self, other: &Matrix) -> Matrix {
        assert_eq!(self.data[0].len(), other.data.len());

        let mut data = vec![vec![c!(0); other.data[0].len()]; self.data.len()];
        for i in 0..self.data.len() {
            for j in 0..other.data[0].len() {
                for k in 0..self.data[0].len() {
                    data[i][j] = data[i][j] + self.data[i][k] * other.data[k][j];
                }
            }
        }
        Matrix { data }
    }

    pub fn dot(&self, other: Matrix) -> C {
        let mut sum = c!(0);
        for i in 0..self.data.len() {
            for j in 0..self.data[0].len() {
                sum = sum + self.data[i][j] * other.data[i][j];
            }
        }
        sum
    }

    pub fn tensor(&self, other: &Matrix) -> Matrix {
        let rows = self.data.len() * other.data.len();
        let cols = self.data[0].len() * other.data[0].len();

        let mut data = vec![vec![c!(0); cols]; rows];

        let nr_rows_other = other.data.len();
        let nr_cols_other = other.data[0].len();

        for i in 0..rows {
            for j in 0..cols {
                let row = i / nr_rows_other;
                let col = j / nr_cols_other;

                let row2 = i % nr_rows_other;
                let col2 = j % nr_cols_other;

                data[i][j] = self.data[row][col] * other.data[row2][col2];
            }
        }
        Matrix { data }
    }

    pub fn norm(&self) -> f64 {
        let mut norm = 0.0;
        for i in 0..self.data.len() {
            for j in 0..self.data[0].len() {
                norm = norm + self.data[i][j].modulus().powf(2.0);
            }
        }
        return norm.sqrt();
    }

    pub fn is_unitary(&self) -> bool {
        let adj = self.adjoint();
        let id = Matrix::identity(self.data.len());
        let res = self.clone() * adj;
        res == id
    }

    pub fn is_hermitian(&self) -> bool {
        self.clone() == self.adjoint()
    }

    pub fn is_vector(&self) -> bool {
        self.data[0].len() == 1
    }

    pub fn size(&self) -> (usize, usize) {
        // (cols, rows)
        (self.data.len(), self.data[0].len())
    }
}

#[macro_export]
macro_rules! mat {
    ($($($a:expr),+);+ $(;)?) => {
        Matrix::new(vec![$(vec![$($a),+]),+])
    };
}

pub fn hadamard() -> Matrix {
    mat![
        c!(1), c!(1);
        c!(1), c!(-1);
    ]
    .scalar_mul(c!(1.0 / 2.0_f64.sqrt()))
}

pub fn cnot() -> Matrix {
    mat![
        c!(1), c!(0), c!(0), c!(0);
        c!(0), c!(1), c!(0), c!(0);
        c!(0), c!(0), c!(0), c!(1);
        c!(0), c!(0), c!(1), c!(0);
    ]
}

pub fn phase_shift(phase: f64) -> Matrix {
    mat![
        c!(1), c!(0);
        c!(0), c!(phase.cos(), phase.sin());
    ]
}

pub fn unitary_modular(a: usize, n: usize) -> Matrix {
    let nbit_size = min_bit_size(n as u32);
    let mbit_size = nbit_size * 2;
    let qbit_size = nbit_size + mbit_size;

    let m_size = (2 as u32).clone().pow(qbit_size.clone() as u32) as usize;
    let n_bit_represenation = (2 as u32).clone().pow(nbit_size.clone() as u32);
    let m_bit_represenation = (2 as u32).clone().pow(mbit_size.clone() as u32);

    let mut matrix = Matrix::zero_sq(m_size);

    for i in 0..m_bit_represenation {
        let f = mod_power(a as u32, i, n as u32) as usize;
        let sq_factor = (i * n_bit_represenation) as usize;
        matrix = matrix.set( sq_factor + f, sq_factor, c!(1));
    }

    matrix
}

pub fn quantum_fourier(n: usize) -> Matrix {
    let size = (2 as u32).clone().pow(n.clone() as u32) as usize;
    let mut matrix = Matrix::zero_sq(size);

    let base = c!((size as f64).powf(-0.5));
    for i in 0..size {
        for j in 0..size {
            let v = c!(0.0, 1.0).pow(i * j);
            matrix = matrix.set(i, j, base * v);
        }
    }

    matrix

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_macro() {
        let m = mat!(c!(1), c!(2); c!(3), c!(4));
        assert_eq!(m.data, vec![vec![c!(1), c!(2)], vec![c!(3), c!(4)]]);
    }

    #[test]
    fn test_matrix_new() {
        let m = Matrix::new(vec![vec![c!(1), c!(2)], vec![c!(3), c!(4)]]);
        assert_eq!(m.data, vec![vec![c!(1), c!(2)], vec![c!(3), c!(4)]]);
    }

    #[test]
    fn test_matrix_identity() {
        let m = Matrix::identity(3);
        assert_eq!(
            m.data,
            vec![
                vec![c!(1), c!(0), c!(0)],
                vec![c!(0), c!(1), c!(0)],
                vec![c!(0), c!(0), c!(1)]
            ]
        );
    }

    #[test]
    fn test_matrix_transpose() {
        let m = mat!(c!(1), c!(2); c!(3), c!(4));
        let m2 = mat!(c!(1), c!(3); c!(2), c!(4));
        let t = m.transpose();
        assert_eq!(t, m2);
    }

    #[test]
    fn test_matrix_add() {
        let m1 = mat!(c!(1), c!(2); c!(3), c!(4));
        let m2 = mat!(c!(5), c!(6); c!(7), c!(8));

        let m3 = m1 + m2;

        let res = mat!(
            c!(6), c!(8);
            c!(10), c!(12);
        );
        assert_eq!(m3, res);
    }

    #[test]
    fn test_matrix_scalar_mul() {
        let m = mat!(c!(1), c!(2); c!(3), c!(4));
        let m2 = m.scalar_mul(c!(2));

        let res = mat!(c!(2), c!(4); c!(6), c!(8));
        assert_eq!(m2, res);
    }

    #[test]
    fn test_matrix_negative_inverse() {
        let m = mat!(c!(1), c!(2); c!(3), c!(4));
        let m2 = m.negative_inverse();

        let res = mat!(c!(-1), c!(-2); c!(-3), c!(-4));
        assert_eq!(m2, res);
    }

    #[test]
    fn test_matrix_mul() {
        let m1 = mat!(c!(1), c!(2); c!(3), c!(4));
        let m2 = mat!(c!(5), c!(6); c!(7), c!(8));

        let m3 = m1 * m2;

        let res = mat!(
            c!(19), c!(22);
            c!(43), c!(50);
        );
        assert_eq!(m3, res);
    }

    #[test]
    fn test_matrix_dot() {
        let m1 = mat!(c!(1), c!(2); c!(3), c!(4));
        let m2 = mat!(c!(5), c!(6); c!(7), c!(8));

        let res = m1.dot(m2);
        assert_eq!(res, c!(70));
    }

    #[test]
    fn test_matrix_norm() {
        let m = mat!(c!(1), c!(2), c!(3)).transpose();
        let res = m.norm();
        assert_eq!(res, 14.0_f64.sqrt());
    }

    #[test]
    fn test_matrix_conjugate() {
        let m = mat!(c!(1, 1), c!(0, 2); c!(3), c!(4, -1));
        let m2 = m.conjugate();

        let res = mat!(c!(1, -1), c!(0, -2); c!(3), c!(4, 1));
        assert_eq!(m2, res);
    }

    #[test]
    fn test_matrix_is_unary() {
        let m = mat!(
            c!(1.0_f64.cos()), c!(1.0_f64.sin() * -1.0), c!(0);
            c!(1.0_f64.sin()), c!(1.0_f64.cos()), c!(0);
            c!(0), c!(0), c!(1);
        );
        assert!(m.is_unitary());

        let m2 = mat!(c!(5), c!(6); c!(7), c!(8));
        assert!(!m2.is_hermitian());
    }

    #[test]
    fn test_matrix_is_hermitian() {
        let m = mat!(
            c!(5), c!(4, 5), c!(6, -16);
            c!(4, -5), c!(13), c!(7);
            c!(6, 16), c!(7), c!(-2.0);
        );

        assert!(m.is_hermitian());

        let m2 = mat!(c!(5), c!(6); c!(7), c!(8));
        assert!(!m2.is_hermitian());
    }

    #[test]
    fn test_matrix_tensor() {
        let m1 = mat!(
            c!(1);
            c!(2);
        );
        let m2 = mat!(
            c!(5);
            c!(6);
            c!(7);
        );

        let m3 = m1.tensor(&m2);

        let res = mat!(
            c!(1) * c!(5);
            c!(1) * c!(6);
            c!(1) * c!(7);

            c!(2) * c!(5);
            c!(2) * c!(6);
            c!(2) * c!(7);
        );
        assert_eq!(m3, res);

        let a0 = c!(1, 2);
        let a1 = c!(0);
        let a2 = c!(1, 10);
        let a3 = c!(1.4, 1.5);

        let b0 = c!(1);
        let b1 = c!(2);
        let b2 = c!(3);
        let b3 = c!(4);
        let b4 = c!(5.1, -1.1);
        let b5 = c!(4, 0);
        let b6 = c!(4);
        let b7 = c!(3, -1);
        let b8 = c!(1.3);
        let b9 = c!(1, 10);

        let m4 = mat!(
            a0, a1;
            a2, a3;
        );

        let m5 = mat!(
            b0, b1, b2;
            b3, b4, b5;
            b6, b7, b8;
            b9, b0, b1;
        );

        let res2 = mat!(
            a0 * b0, a0 * b1, a0 * b2, a1 * b0, a1 * b1, a1 * b2;
            a0 * b3, a0 * b4, a0 * b5, a1 * b3, a1 * b4, a1 * b5;
            a0 * b6, a0 * b7, a0 * b8, a1 * b6, a1 * b7, a1 * b8;
            a0 * b9, a0 * b0, a0 * b1, a1 * b9, a1 * b0, a1 * b1;
            a2 * b0, a2 * b1, a2 * b2, a3 * b0, a3 * b1, a3 * b2;
            a2 * b3, a2 * b4, a2 * b5, a3 * b3, a3 * b4, a3 * b5;
            a2 * b6, a2 * b7, a2 * b8, a3 * b6, a3 * b7, a3 * b8;
            a2 * b9, a2 * b0, a2 * b1, a3 * b9, a3 * b0, a3 * b1;
        );
        assert_eq!(m4.tensor(&m5), res2);
    }

    #[test]
    fn test_matrix_is_vector() {
        let m = mat!(c!(1), c!(2), c!(3));
        assert!(!m.is_vector());
        assert!(m.transpose().is_vector());

        let m2 = mat!(c!(1), c!(2); c!(3), c!(4));
        assert!(!m2.is_vector());
    }

    #[test]
    fn test_arb_matrix_mult() {
        let vec = mat!(c!(5); c!(0); c!(5); c!(0); c!(5); c!(0); c!(5); c!(0));
        let mat = Matrix::zero_sq(8);

        let mat = mat.set(0, 0, c!(1));
        let mat = mat.set(3, 2, c!(1));
        let mat = mat.set(4, 4, c!(1));
        let mat = mat.set(7, 6, c!(1));

        assert_eq!(
            mat * vec,
            mat!(c!(5); c!(0); c!(0); c!(5); c!(5); c!(0); c!(0); c!(5))
        );
    }

    #[test]
    fn test_unitary_modular() {
        let a = 2;
        let n = 3;
        let m = unitary_modular(a, n);

        assert_eq!(m.size(), (64, 64));
        assert_eq!(m.data[1][0], c!(1));
        assert_eq!(m.data[62][60], c!(1));

        let mut vec = Matrix::zero(64, 1);
        for i in 0..16 {
            vec = vec.set(i * 4, 0, c!(5));
        }

        assert_eq!(vec.data[0][0], c!(5));
        assert_eq!(vec.data[4][0], c!(5));
        assert_eq!(vec.data[5][0], c!(0));
        assert_eq!(vec.data[60][0], c!(5));

        let unitary_apply = m * vec;

        assert_eq!(unitary_apply.data[1][0], c!(5));
        assert_eq!(unitary_apply.data[6][0], c!(5));
        assert_eq!(unitary_apply.data[8][0], c!(0));
        assert_eq!(unitary_apply.data[9][0], c!(5));
        assert_eq!(unitary_apply.data[10][0], c!(0));
        assert_eq!(unitary_apply.data[11][0], c!(0));
        assert_eq!(unitary_apply.data[62][0], c!(5));
    }


    #[test]
    fn tetst_qft() {
        let m = quantum_fourier(2);

        let half = c!(0.5);

        let res = mat![
            c!(1),c!(1),c!(1),c!(1);
            c!(1),c!(0, 1),c!(-1),c!(0, -1);
            c!(1),c!(-1),c!(1),c!(-1);
            c!(1),c!(0, -1),c!(-1),c!(0, 1);
        ].scalar_mul(half);

        assert_eq!(m, res);
    }
}
