mod complex;
mod double_slit;
mod matrix;

mod quantum_sim;

mod quantum_assembler_lexer;
mod quantum_assembler_parser;
mod quantum_assembler_executor;

mod util;

use crate::{complex::*, matrix::*, quantum_sim::*};

fn main() {
    let halfsqrt2 = c!(0.5 * 2.0_f64.sqrt());
    let ket = mat![c!(1); c!(0, 1);].scalar_mul(halfsqrt2);
    let transform = mat![c!(1), c!(0, -1); c!(0, 1), c!(2);];

    println!("Hello, world!");
}
