use std::collections::HashMap;

mod lexer;
mod parser;
mod quantum_sim;

mod executor;

pub enum QuantumSimError {
    RuntimeError(executor::RunTimeError),
    ParseError(parser::ParseError),
}

pub fn run(
    input: String,
) -> Result<HashMap<String, (crate::matrix::matrix::Matrix, String)>, QuantumSimError> {
    let ast = parser::parse(input);
    if ast.is_err() {
        return Err(QuantumSimError::ParseError(ast.err().unwrap()));
    }

    let result = executor::execute_script(ast.unwrap());
    if result.is_err() {
        return Err(QuantumSimError::RuntimeError(result.err().unwrap()));
    }

    Ok(result.unwrap())
}
