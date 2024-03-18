use crate::{c, complex::*, mat, matrix::*};

fn slits_to_matrix_size(slits: usize) -> usize {
    // 1 for the start, slits for the slits, slits * 2 +1 for targets
    1 + slits + ((slits * 2) + 1)
}

fn setup_prob_matrix(slits: usize) -> Matrix {
    let mut B = Matrix::identity(slits_to_matrix_size(slits));
    let thrith = c!(1.0 / 3.0);
    B.data[0][0] = c!(0.0);

    for i in 0..slits {
        print!("{}", i);
        B.data[i + 1][0] = c!(1.0 / slits as f64);
        B.data[i + 1][i + 1] = c!(0);

        B.data[slits + (i * 2) + 1][i + 1] = thrith;
        B.data[slits + (i * 2) + 2][i + 1] = thrith;
        B.data[slits + (i * 2) + 3][i + 1] = thrith;
    }

    B
}

fn prob_double_slit(slits: usize, ) -> (Matrix, Matrix) {
    let B = setup_prob_matrix(slits);
    let transform = B.clone() * B.clone();
    let mut x = Matrix::zero_sq(slits_to_matrix_size(slits));
    x.data[0][0] = c!(1.0);

    (transform.clone(), transform * x)
}

fn setup_quantum_matrix(slits: usize) -> Matrix {
    let mut B = Matrix::identity(slits_to_matrix_size(slits));

    B.data[0][0] = c!(0.0);

    for i in 0..slits {
        print!("{}", i);
        B.data[i + 1][0] = c!(1.0 / (slits as f64).sqrt());
        B.data[i + 1][i + 1] = c!(0);

        B.data[slits + (i * 2) + 1][i + 1] = c!(1.0 / 6.0_f64.sqrt(), 1.0 / 6.0_f64.sqrt());
        B.data[slits + (i * 2) + 2][i + 1] = c!(1.0 / 6.0_f64.sqrt(), -1.0 / 6.0_f64.sqrt());
        B.data[slits + (i * 2) + 3][i + 1] = c!(-1.0 / 6.0_f64.sqrt(), -1.0 / 6.0_f64.sqrt());
    }

    B
}

fn quantum_double_slit(slits: usize, ) -> (Matrix, Matrix) {
    let B = setup_quantum_matrix(slits);
    let transform = B.clone() * B.clone();
    let mut x = Matrix::zero(1 + slits + (slits * 2) + 1, 1);
    x.data[0][0] = c!(1.0);

    (transform.clone(), transform * x)
}
fn bool_double_slit(start: Matrix, transformation: Matrix, steps: u32) -> Matrix {
    let mut state = start;
    for _ in 0..steps {
        state = transformation.clone() * state;
    }
    state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool_double_slit() {
        let state0 = mat![c!(0); c!(1); c!(0);];

        let transformation = mat![
        c!(0), c!(0), c!(1);
        c!(1), c!(0), c!(0);
        c!(0), c!(1), c!(0);
        ];

        assert_eq!(
            bool_double_slit(state0.clone(), transformation.clone(), 1),
            mat![c!(0); c!(0); c!(1);]
        );
        assert_eq!(
            bool_double_slit(state0.clone(), transformation.clone(), 2),
            mat![c!(1); c!(0); c!(0);]
        );
        assert_eq!(
            bool_double_slit(state0.clone(), transformation, 3),
            mat![c!(0); c!(1); c!(0);]
        );
    }

    #[test]
    fn test_generate_prob_matrix() {
        let B = setup_prob_matrix(2);

        let half = c!(1.0 / 2.0);
        let thirth = c!(1.0 / 3.0);

        let expected = mat!(
            c!(0), c!(0), c!(0), c!(0), c!(0), c!(0), c!(0), c!(0);
            half, c!(0), c!(0), c!(0), c!(0), c!(0), c!(0), c!(0);
            half, c!(0), c!(0), c!(0), c!(0), c!(0), c!(0), c!(0);
            c!(0), thirth, c!(0), c!(1), c!(0), c!(0), c!(0), c!(0);
            c!(0), thirth, c!(0), c!(0), c!(1), c!(0), c!(0), c!(0);
            c!(0), thirth, thirth, c!(0), c!(0), c!(1), c!(0), c!(0);
            c!(0), c!(0), thirth, c!(0), c!(0), c!(0), c!(1), c!(0);
            c!(0), c!(0), thirth, c!(0), c!(0), c!(0), c!(0), c!(1);
        );

        assert_eq!(B, expected);

        let B2 = setup_prob_matrix(1);

        let expected2 = mat!(
            c!(0), c!(0), c!(0), c!(0), c!(0);
            c!(1), c!(0), c!(0), c!(0), c!(0);
            c!(0), thirth, c!(1), c!(0), c!(0);
            c!(0), thirth, c!(0), c!(1), c!(0);
            c!(0), thirth, c!(0), c!(0), c!(1);
        );

        assert_eq!(B2, expected2);
    }

    #[test]
    fn test_prob_double_slit() {
        let (transform, x) = prob_double_slit(2);

        let sixth = c!(1.0 / 6.0);
        let thirth = c!(1.0 / 3.0);
        let expected = mat![c!(0); c!(0); c!(0); sixth; sixth; thirth; sixth; sixth;];

        assert_eq!(x, expected);
    }

    #[test]
    fn test_quantum_double_slit() {
        let (transform, x) = quantum_double_slit(2);
        assert_eq!(x.data[5][0], c!(0));

        let (transform, x2) = quantum_double_slit(4);
        assert_eq!(x2.data[7][0], c!(0));
        assert_eq!(x2.data[9][0], c!(0));
        assert_eq!(x2.data[11][0], c!(0));
    }
}
