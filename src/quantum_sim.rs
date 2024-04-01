use crate::{c, complex::*, mat, matrix::*};

pub fn prob_at(m: &Matrix, idx: usize) -> f64 {
    if (idx >= m.data.len()) || (m.data[0].len() != 1) {
        panic!("Invalid index");
    }

    let norm = m.norm();
    let val = m.data[idx][0].modulus();

    val.powf(2.0) / norm.powf(2.0)
}

pub fn transisition_amplitude(from: &Matrix, to: &Matrix) -> C {
    // TODO: Check if the matrices are compatible
    let adj = to.adjoint();
    let res = &adj * from;
    res.data[0][0]
}

pub fn apply_transformation(ket: &Matrix, transform: &Matrix) -> Matrix {
    if (!transform.is_hermitian() || !ket.is_vector() || ket.data.len() != transform.data.len()) {
        panic!("Invalid input");
    }

    transform * ket
}

pub fn mean_transistion(ket: &Matrix, transform: &Matrix) -> C {
    let res = apply_transformation(ket, transform);

    transisition_amplitude(ket, &res)
}

pub fn variance_transistion(ket: &Matrix, transform: &Matrix) -> C {
    let mu = mean_transistion(ket, transform);
    let mu_marix = Matrix::identity(ket.data.len()).scalar_mul(mu);

    let var_marix = transform.clone() + mu_marix.scalar_mul(c!(-1.0));

    let var_matrix_squared = &var_marix * &var_marix;
    let res = ket.transpose() * var_matrix_squared * ket.clone();

    res.data[0][0]
}

fn parse_quantum_program(program: String, variables: Vec<Matrix>) -> Matrix {


    return mat![c!(0)]
}

#[cfg(test)]
mod tests {
    use crate::util::f64_equal;

    use super::*;

    #[test]
    fn test_prob() {
        let vec = mat![c!(-3, -1); c!(0, -2); c!(0, 1); c!(2);];

        assert!((vec.norm() - 4.3589).abs() < 0.0001);
        assert!((prob_at(&vec, 2) - 0.052624).abs() < 0.0001);

        let ket1 = mat![c!(1, 1); c!(0, 1)];
        let ket2 = mat![c!(2, 4); c!(-1, 3)];

        let diff1 = prob_at(&ket1, 1) - prob_at(&ket2, 1);
        let diff2 = prob_at(&ket1, 0) - prob_at(&ket2, 0);

        assert!(diff1.abs() < 0.00000001);
        assert!(diff2.abs() < 0.00000001);
    }

    #[test]
    fn test_matrix_normalized() {
        let m = mat![c!(1), c!(2), c!(3)].transpose();
        let res = m.normalized();

        assert_eq!(res.norm(), 1.0);
        assert!((prob_at(&res, 0) - prob_at(&m, 0)).abs() < 0.0000001);
    }

    #[test]
    fn test_transition_amplitude() {
        let halfsqrt2 = c!(0.5 * 2.0_f64.sqrt());
        let m1 = mat!(c!(1.0) ; c!(0.0, 1.0 )).scalar_mul(halfsqrt2);
        let m2 = mat!(c!(0.0, 1.0) ; c!(-1.0 )).scalar_mul(halfsqrt2);

        let res = transisition_amplitude(&m1, &m2);

        assert_eq!(res.a, 0.0);
        assert!(f64_equal(res.b, -1.0));
    }

    #[test]
    fn test_observers() {
        let halfsqrt2 = c!(0.5 * 2.0_f64.sqrt());

        let ket = mat![c!(1); c!(0, 1);].scalar_mul(halfsqrt2);
        let transform = mat![c!(1), c!(0, -1); c!(0, 1), c!(2);];

        let res = apply_transformation(&ket, &transform);

        assert!(f64_equal(res.data[0][0].a, 2.0_f64.sqrt()));
        assert_eq!(res.data[1][0], c!(0.0, 1.5 * 2.0_f64.sqrt()));

        assert!(f64_equal(mean_transistion(&ket, &transform).a, 2.5));
        assert!(f64_equal(variance_transistion(&ket, &transform).a, 1.0));
    }
}
