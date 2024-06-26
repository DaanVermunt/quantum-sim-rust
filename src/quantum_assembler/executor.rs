use std::{collections::HashMap, error, f64::consts::PI, fmt};

use crate::{
    c,
    matrix::{complex::C, matrix::{cnot, hadamard, phase_shift, quantum_fourier, unitary_modular, Matrix}},
};

use super::{
    parser::{ASTNode, MemoryLocation, AST},
    quantum_sim::{measure_partial_vec, measure_vec, qbit_length},
};

#[derive(Debug)]
pub enum RunTimeError {
    SyntaxError(String), // TOO GENERIC
    NotImplemented,
}

impl fmt::Display for RunTimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RunTimeError::SyntaxError(mess) => write!(f, "Syntax error: {}", mess),
            RunTimeError::NotImplemented => write!(f, "Not implemented"),
        }
    }
}

impl error::Error for RunTimeError {
    fn description(&self) -> &str {
        match self {
            RunTimeError::SyntaxError(_) => "Syntax error in code",
            RunTimeError::NotImplemented => "Not implemented",
        }
    }
}

type Heap = HashMap<String, LiteralValue>;
type Measurements = HashMap<String, (Matrix, String)>;

#[derive(Debug)]
struct QuantumMemory {
    heap: Heap,
    measurements: Measurements,
}

#[derive(Debug, Clone, PartialEq)]
enum LiteralValue {
    Matrix(Matrix),
    Int(i32),

    Selection(String, MemoryLocation, i32, i32),

    Measurement(Matrix, String),
}

fn unwrap_matrix(lit: &LiteralValue) -> Result<&Matrix, RunTimeError> {
    match lit {
        LiteralValue::Matrix(m) => Ok(m),
        _ => Err(RunTimeError::SyntaxError("Invalid matrix".to_string())),
    }
}

fn unwrap_selection(
    lit: &LiteralValue,
) -> Result<(&String, &MemoryLocation, &i32, &i32), RunTimeError> {
    match lit {
        LiteralValue::Selection(key, mem, from, to) => Ok((key, mem, from, to)),
        _ => Err(RunTimeError::SyntaxError("Invalid matrix".to_string())),
    }
}

fn unwrap_int(lit: &LiteralValue) -> Result<&i32, RunTimeError> {
    match lit {
        LiteralValue::Int(m) => Ok(m),
        _ => Err(RunTimeError::SyntaxError("Invalid matrix".to_string())),
    }
}

fn validate_param_len(
    params: &Vec<(String, LiteralValue)>,
    expected: usize,
) -> Result<(), RunTimeError> {
    if params.len() != expected {
        return Err(RunTimeError::SyntaxError(
            "Invalid number of parameters".to_string(),
        ));
    }

    Ok(())
}

fn parse_params_from_prefebs(lit: &String, expected: usize) -> Result<Vec<usize>, RunTimeError> {
    let re = regex::Regex::new(r"\d+");

    if re.is_err() {
        return Err(RunTimeError::SyntaxError("Invalid literal".to_string()));
    }

    let re = re.unwrap();

    let nmbrs: Vec<usize> = re
        .find_iter(lit)
        .map(|m| m.as_str().parse::<usize>().unwrap())
        .collect();

    if nmbrs.len() != expected {
        return Err(RunTimeError::SyntaxError("Invalid literal".to_string()));
    }

    Ok(nmbrs)
}

fn parse_literal(v: &String) -> Result<LiteralValue, RunTimeError> {
    match v.as_str() {
        "G_H" => Ok(LiteralValue::Matrix(hadamard())),
        "G_CNOT" => Ok(LiteralValue::Matrix(cnot())),
        _ => {
            if v.starts_with("G_R_") {
                let nmbrs = parse_params_from_prefebs(v, 1).unwrap();
                return Ok(LiteralValue::Matrix(phase_shift(PI / (nmbrs[0] as f64))));
            }
            if v.starts_with("G_I_") {
                let nmbrs = parse_params_from_prefebs(v, 1).unwrap();
                return Ok(LiteralValue::Matrix(Matrix::identity(nmbrs[0])));
            }
            if v.starts_with("G_Uf_") {
                let nmbrs = parse_params_from_prefebs(v, 2).unwrap();
                return Ok(LiteralValue::Matrix(unitary_modular(nmbrs[0], nmbrs[1])));
            }
            if v.starts_with("G_QFTI_") {
                let nmbrs = parse_params_from_prefebs(v, 1).unwrap();
                return Ok(LiteralValue::Matrix(quantum_fourier(nmbrs[0]).adjoint()));
            }
            if v.parse::<i32>().is_ok() {
                return Ok(LiteralValue::Int(v.parse::<i32>().unwrap()));
            }
            Err(RunTimeError::SyntaxError("Invalid literal".to_string()))
        }
    }
}

fn parse_identifier(
    var_name: &String,
    memory: &QuantumMemory,
) -> Result<LiteralValue, RunTimeError> {
    match memory.heap.get(var_name) {
        Some(val) => Ok(val.clone()),
        None => Err(RunTimeError::SyntaxError("Variable not found".to_string())),
    }
}

fn parse_var_assignment(
    var_name: &String,
    val: &ASTNode,
    memory_loc: &MemoryLocation,
    memory: &mut QuantumMemory,
) -> Result<Option<LiteralValue>, RunTimeError> {
    let val = execute_ast_node(val, memory).unwrap();
    match val {
        Some(val) => {
            match (memory_loc, val.clone()) {
                (MemoryLocation::Heap, (_, LiteralValue::Int(_))) => {
                    memory.heap.insert(var_name.clone(), val.1);
                }
                (MemoryLocation::Heap, (_, LiteralValue::Matrix(_))) => {
                    memory.heap.insert(var_name.clone(), val.1);
                }
                (MemoryLocation::Heap, (_, LiteralValue::Selection(_, _, _, _))) => {
                    memory.heap.insert(var_name.clone(), val.1);
                }
                (MemoryLocation::Measurement, (_, LiteralValue::Measurement(a, b))) => {
                    memory.measurements.insert(var_name.clone(), (a, b));
                }
                _ => return Err(RunTimeError::SyntaxError("Invalid assignment".to_string())),
            };
            Ok(None)
        }
        None => Err(RunTimeError::SyntaxError("Variable not found".to_string())),
    }
}

fn parse_func_application(
    func: &String,
    params: &Vec<ASTNode>,
    memory: &mut QuantumMemory,
) -> Result<Option<(String, LiteralValue)>, RunTimeError> {
    let params = params
        .iter()
        .map(|p| execute_ast_node(p, memory).unwrap())
        .filter_map(|p| p)
        .collect::<Vec<(String, LiteralValue)>>();

    match &func[..] {
        "INITIALIZE" => {
            validate_param_len(&params, 1).unwrap();

            let value = unwrap_int(&params[0].1).unwrap();

            let matrix = Matrix::zero((2 as u32).clone().pow(value.clone() as u32) as usize, 1);
            Ok(Some((
                func.clone(),
                LiteralValue::Matrix(matrix.set(0, 0, c!(1))),
            )))
        }
        "INVERSE" => {
            validate_param_len(&params, 1).unwrap();

            let matrix = unwrap_matrix(&params[0].1).unwrap();

            if !matrix.is_hermitian() {
                return Err(RunTimeError::SyntaxError(
                    "Input invalid for INVERSE, should be a hermetian matrix".to_string(),
                ));
            }

            Ok(Some((func.clone(), LiteralValue::Matrix(matrix.adjoint()))))
        }
        "TENSOR" => {
            validate_param_len(&params, 2).unwrap();

            let matrix1 = unwrap_matrix(&params[0].1).unwrap();
            let matrix2 = unwrap_matrix(&params[1].1).unwrap();

            Ok(Some((
                func.clone(),
                LiteralValue::Matrix(matrix1.tensor(matrix2)),
            )))
        }
        "CONCAT" => {
            validate_param_len(&params, 2).unwrap();

            let matrix1 = unwrap_matrix(&params[0].1).unwrap();
            let matrix2 = unwrap_matrix(&params[1].1).unwrap();

            if matrix1.size() != matrix2.size() {
                return Err(RunTimeError::SyntaxError(
                    "Matrix sizes should be equal to CONCAT".to_string(),
                ));
            }

            Ok(Some((
                func.clone(),
                LiteralValue::Matrix(matrix1 * matrix2),
            )))
        }
        "APPLY" => {
            validate_param_len(&params, 2).unwrap();

            let matrix = unwrap_matrix(&params[0].1).unwrap();
            let vector = unwrap_matrix(&params[1].1).unwrap();

            if !vector.is_vector() || vector.size().0 != matrix.size().1 {
                println!("Vector{:?} x Matrix{:?}, herm({})", vector.size(), matrix.size(), matrix.is_hermitian());
                return Err(RunTimeError::SyntaxError(
                    "Input invalid for APPLY, first arg should be a hermetian matrix & the second arg should be vector with equal columns".to_string(),
                ));
            }

            Ok(Some((func.clone(), LiteralValue::Matrix(matrix * vector))))
        }
        "SELECT" => {
            validate_param_len(&params, 3).unwrap();

            let key = params[0].0.clone();
            let vector = unwrap_matrix(&params[0].1).unwrap();
            let start = unwrap_int(&params[1].1).unwrap();
            let end = unwrap_int(&params[2].1).unwrap();

            let qbit_len = qbit_length(vector);
            if !vector.is_vector() || start > end || (*end as usize) > qbit_len {
                return Err(RunTimeError::SyntaxError(
                    "Invalid range for SELECT".to_string(),
                ));
            }

            Ok(Some((
                func.clone(),
                LiteralValue::Selection(
                    key.clone(),
                    MemoryLocation::Heap,
                    start.clone(),
                    end.clone(),
                ),
            )))
        }
        "MEASURE" => {
            validate_param_len(&params, 1).unwrap();

            let vec = unwrap_matrix(&params[0].1);

            if vec.is_ok() {
                let vec = vec.unwrap();
                if !vec.is_vector() {
                    return Err(RunTimeError::SyntaxError(
                        "Invalid input for MEASURE, should be a vector".to_string(),
                    ));
                }

                return Ok(Some((
                    func.clone(),
                    LiteralValue::Measurement(vec.clone(), measure_vec(vec)),
                )));
            }

            let (key, _, from, to) = unwrap_selection(&params[0].1).unwrap();
            let matrix = memory.heap.get(key).unwrap().clone();
            let vec = unwrap_matrix(&matrix).unwrap();

            if !vec.is_vector() {
                return Err(RunTimeError::SyntaxError(
                    "Invalid input for MEASURE, should be a vector".to_string(),
                ));
            }

            let res = measure_partial_vec(vec, *from, *to);

            memory
                .heap
                .insert(key.clone(), LiteralValue::Matrix(res.clone()));

            Ok(Some((
                func.clone(),
                LiteralValue::Measurement(res.clone(), measure_vec(&res)),
            )))
        }
        _ => Err(RunTimeError::NotImplemented),
    }
}

fn execute_ast_node(
    ast_node: &ASTNode,
    memory: &mut QuantumMemory,
) -> Result<Option<(String, LiteralValue)>, RunTimeError> {
    match ast_node {
        ASTNode::Literal(val) => Ok(Some(("_".to_string(), parse_literal(val).unwrap()))),
        ASTNode::Identifier(var_name) => Ok(Some((
            var_name.clone(),
            parse_identifier(var_name, memory).unwrap(),
        ))),
        ASTNode::VariableAssignment(var_name, memory_loc, val) => {
            parse_var_assignment(var_name, &*val, memory_loc, memory).unwrap();
            Ok(None)
        }
        ASTNode::FunctionApplication(func, params) => parse_func_application(func, params, memory),
    }
}

pub fn execute_script(ast: AST) -> Result<HashMap<String, (Matrix, String)>, RunTimeError> {
    let heap = HashMap::<String, LiteralValue>::new();
    let measurements = HashMap::<String, (Matrix, String)>::new();

    let mut memory = QuantumMemory { heap, measurements };

    // LOOP TROUGH AST AND RUN
    for node in ast {
        // println!("{:?}", node);
        // println!("{:?}", memory.heap);
        execute_ast_node(&node, &mut memory).unwrap();
    }

    Ok(memory.measurements)
}

#[cfg(test)]
mod tests {
    use crate::{mat, quantum_assembler::parser::parse};

    use super::*;

    #[test]
    fn test_init_and_measure_executor() {
        let ast = parse(
            "
        INITIALIZE R 2
        MEASURE R RES
        "
            .to_string(),
        );
        assert!(ast.is_ok());

        let res = execute_script(ast.unwrap());

        assert!(res.is_ok());

        let res = res.unwrap();
        assert!(res.contains_key("RES"));
        assert_eq!(res.get("RES").unwrap().0, mat![c!(1); c!(0); c!(0); c!(0)]);
        assert_eq!(res.get("RES").unwrap().1, "00");
    }

    #[test]
    fn test_tensor_hadamar_and_apply() {
        let ast = parse(
            "
        INITIALIZE R 2
        U TENSOR G_H G_H
        APPLY U R
        MEASURE R RES
        "
            .to_string(),
        );
        assert!(ast.is_ok());

        let res = execute_script(ast.unwrap());

        assert!(res.is_ok());

        let res = res.unwrap();
        assert!(res.contains_key("RES"));
        assert_eq!(
            res.get("RES").unwrap().0,
            mat![c!(0.5); c!(0.5);c!(0.5);c!(0.5)]
        );
    }

    #[test]
    fn test_select() {
        let ast = parse(
            "
            INITIALIZE R 2
            U TENSOR G_H G_I_2
            APPLY U R
            SELECT S1 R 0 1
            MEASURE S1 RES1
            APPLY G_CNOT R
            MEASURE R RES2
        "
            .to_string(),
        );
        assert!(ast.is_ok());

        let res = execute_script(ast.unwrap());

        assert!(res.is_ok());

        let res = res.unwrap();
        assert!(res.contains_key("RES2"));
        let res2 = res.get("RES2").unwrap();
        assert!(res2.1 == "11" || res2.1 == "00");
    }
}
