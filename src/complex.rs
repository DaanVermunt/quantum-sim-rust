use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct C {
    pub a: f64,
    pub b: f64,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CPolar {
    r: f64,
    t: f64,
}

impl Add for C {
    type Output = C;

    fn add(self, other: C) -> C {
        C {
            a: self.a + other.a,
            b: self.b + other.b,
        }
    }
}

impl Sub for C {
    type Output = C;

    fn sub(self, other: C) -> C {
        C {
            a: self.a - other.a,
            b: self.b - other.b,
        }
    }
}

impl Div for C {
    type Output = C;

    fn div(self, other: C) -> C {
        C {
            a: (self.a * other.a + self.b * other.b) / (other.a * other.a + other.b * other.b),
            b: (self.b * other.a - self.a * other.b) / (other.a * other.a + other.b * other.b),
        }
    }
}

impl Mul for C {
    type Output = C;

    fn mul(self, other: C) -> C {
        C {
            a: self.a * other.a + self.b * other.b * -1.0,
            b: self.a * other.b + self.b * other.a,
        }
    }
}

impl C {
    pub fn new<T: Into<f64> + Copy>(a: T, b: T) -> C {
        C {
            a: a.into(),
            b: b.into(),
        }
    }

    pub fn from_polar(p: CPolar) -> C {
        C {
            a: p.r * p.t.cos(),
            b: p.r * p.t.sin(),
        }
    }

    pub fn modulus(self) -> f64 {
        (self.a * self.a + self.b * self.b).sqrt()
    }

    pub fn conjugate(self) -> C {
        C {
            a: self.a,
            b: self.b * -1.0,
        }
    }

    pub fn sqrt(self) -> C {
        let r = self.modulus();
         C::new(r.sqrt(), 0.0) * (self + C::new(r, 0.0)) / C::new((self + C::new(r, 0.0)).modulus(), 0.0)
    }

    pub fn to_polar(self) -> CPolar {
        CPolar {
            r: self.modulus(),
            t: (self.b / self.a).atan(),
        }
    }
}

#[macro_export]
macro_rules! c {
    ($a: expr) => {
        C::new($a as f64, 0.0)
    };
    ($a: expr, $b:expr) => {
        C::new($a, $b)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_macr() {
        assert_eq!(c!(1, 1), c!(1, 1));
        assert_eq!(c!(1, 0), c!(1));
    }

    #[test]
    fn equality() {
        assert_eq!(c!(1, 1), c!(1, 1));
        assert_ne!(c!(1, 1), c!(1, 2));
        assert_ne!(c!(2, 1), c!(1, 2));
    }

    #[test]
    fn add() {
        assert_eq!(c!(1, 1) + c!(1, 1), c!(2, 2));
        assert_eq!(c!(1, 1) + c!(1, 2), c!(2, 3));
        assert_eq!(c!(2, 1) + c!(1, 2), c!(3, 3));
    }

    #[test]
    fn mul() {
        assert_eq!(c!(3, -1) * c!(1, 4), c!(7, 11));
    }

    #[test]
    fn sub() {
        assert_eq!(c!(1, 1) - c!(1, 1), c!(0, 0));
        assert_eq!(c!(1, 1) - c!(1, 2), c!(0, -1));
        assert_eq!(c!(2, 1) - c!(1, 2), c!(1, -1));
    }

    #[test]
    fn div() {
        assert_eq!(c!(-2, 1) / c!(1, 2), c!(0, 1));
    }

    #[test]
    fn modulus() {
        assert_eq!(c!(1, -1).modulus(), 2.0_f64.sqrt());
    }

    #[test]
    fn conjugate() {
        assert_eq!(c!(1, -1).conjugate(), c!(1, 1));
        assert_eq!(c!(1, 1).conjugate(), c!(1, -1));
    }

    #[test]
    fn to_polar() {
        assert_eq!(
            c!(1, 1).to_polar(),
            CPolar {
                r: 2.0_f64.sqrt(),
                t: 0.25 * std::f64::consts::PI
            }
        );
    }

    #[test]
    fn from_polar() {
        assert_eq!(C::from_polar(c!(2, 1).to_polar()), c!(2, 1));
    }

    #[test]
    fn test_sqrt() {
        let root = c!(0, 9).sqrt();
        assert!(root.a - 2.12 < 0.01);
        assert!(root.b - 2.12 < 0.01);
    }
}
