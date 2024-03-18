use crate::{c, complex::*};

use std::ops::{Add, Mul};

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

impl Mul for Matrix {
    type Output = Matrix;

    fn mul(self, other: Matrix) -> Matrix {
        assert_eq!(self.data[0].len(), other.data.len());

        let mut data = vec![vec![c!(0); other.data[0].len()]; self.data.len()];
        for i in 0..self.data.len() {
            for j in 0..other.data[0].len() {
                for k in 0..self.data[0].len() {
                    data[i][j] = data[i][j] + self.data[i][k] * other.data[k][j];
                }
            }
        }
        Matrix { data: data }
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
        let mut data = vec![vec![c!(0); cols]; rows];
        Matrix { data: data }
    }

    pub fn identity(size: usize) -> Matrix {
        let mut data = vec![vec![c!(0); size]; size];
        for i in 0..size {
            data[i][i] = c!(1);
        }
        Matrix { data: data }
    }

    pub fn transpose(&self) -> Matrix {
        let mut data = vec![vec![c!(0); self.data.len()]; self.data[0].len()];
        for i in 0..self.data.len() {
            for j in 0..self.data[0].len() {
                data[j][i] = self.data[i][j];
            }
        }
        Matrix { data: data }
    }

    pub fn conjugate(&self) -> Matrix {
        let mut data = vec![vec![c!(0); self.data.len()]; self.data[0].len()];
        for i in 0..self.data.len() {
            for j in 0..self.data[0].len() {
                data[i][j] = self.data[i][j].conjugate();
            }
        }
        Matrix { data: data }
    }

    pub fn adjoint(&self) -> Matrix {
        self.conjugate().transpose()
    }

    pub fn negative_inverse(&self) -> Matrix {
        let mut data = vec![vec![c!(0); self.data.len()]; self.data[0].len()];
        for i in 0..self.data.len() {
            for j in 0..self.data[0].len() {
                data[i][j] = c!(-1) * self.data[i][j];
            }
        }
        Matrix { data: data }
    }

    pub fn scalar_mul(&self, scalar: C) -> Matrix {
        let mut data = vec![vec![c!(1); self.data.len()]; self.data[0].len()];
        for i in 0..self.data.len() {
            for j in 0..self.data[0].len() {
                data[i][j] = self.data[i][j] * scalar;
            }
        }
        Matrix { data: data }
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

    pub fn tensor(&self, other: Matrix) -> Matrix {
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
        Matrix { data: data }
    }

    pub fn norm(&self) -> C {
        self.dot(self.clone()).sqrt()
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
}

#[macro_export]
macro_rules! mat {
    ($($($a:expr),+);+ $(;)?) => {
        Matrix::new(vec![$(vec![$($a),+]),+])
    };
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
        assert_eq!(res.a, 14.0_f64.sqrt());
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

        let m3 = m1.tensor(m2);

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
        assert_eq!(m4.tensor(m5), res2);
    }
}
