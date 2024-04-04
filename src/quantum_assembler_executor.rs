use std::{collections::HashMap, error, f64::consts::PI, fmt, vec};

use crate::{
    c, cnot, hadamard, mat, matrix, measure_partial_vec, measure_vec, phase_shift, qbit_length,
    quantum_assembler_parser::{ASTNode, MemoryLocation, AST},
    Matrix, C,
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
            RunTimeError::SyntaxError(mess) => "Syntax error in code",
            RunTimeError::NotImplemented => "Not implemented",
        }
    }
}

type Heap = HashMap<String, LiteralValue>;
type Measurements = HashMap<String, (Matrix, String)>;
type Selection = HashMap<String, (String, MemoryLocation, i32, i32)>;

#[derive(Debug)]
pub struct QuantumMemory {
    heap: Heap,
    measurements: Measurements,
    selections: Selection,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Matrix(Matrix),
    Int(i32),

    Selection(String, MemoryLocation, i32, i32),

    Measurement(Matrix, String),
}

pub fn unwrap_matrix(lit: &LiteralValue) -> Result<&Matrix, RunTimeError> {
    match lit {
        LiteralValue::Matrix(m) => Ok(m),
        _ => Err(RunTimeError::SyntaxError("Invalid matrix".to_string())),
    }
}

pub fn unwrap_selection(
    lit: &LiteralValue,
) -> Result<(&String, &MemoryLocation, &i32, &i32), RunTimeError> {
    match lit {
        LiteralValue::Selection(key, mem, from, to) => Ok((key, mem, from, to)),
        _ => Err(RunTimeError::SyntaxError("Invalid matrix".to_string())),
    }
}

pub fn unwrap_int(lit: &LiteralValue) -> Result<&i32, RunTimeError> {
    match lit {
        LiteralValue::Int(m) => Ok(m),
        _ => Err(RunTimeError::SyntaxError("Invalid matrix".to_string())),
    }
}

pub fn validate_param_len(
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

pub fn parse_literal(v: &String) -> Result<LiteralValue, RunTimeError> {
    match &v[..] {
        "G_H" => Ok(LiteralValue::Matrix(hadamard())),
        "G_R_2" => Ok(LiteralValue::Matrix(phase_shift(PI / 2.0))),
        "G_R_4" => Ok(LiteralValue::Matrix(phase_shift(PI / 4.0))),
        "G_I_2" => Ok(LiteralValue::Matrix(Matrix::identity(2))),
        "G_I_4" => Ok(LiteralValue::Matrix(Matrix::identity(4))),
        "G_I_8" => Ok(LiteralValue::Matrix(Matrix::identity(8))),
        "G_CNOT" => Ok(LiteralValue::Matrix(cnot())),
        _ => {
            if v.parse::<i32>().is_ok() {
                return Ok(LiteralValue::Int(v.parse::<i32>().unwrap()));
            }
            Err(RunTimeError::SyntaxError("Invalid literal".to_string()))
        }
    }
}

pub fn parse_identifier(
    var_name: &String,
    memory: &QuantumMemory,
) -> Result<LiteralValue, RunTimeError> {
    match memory.heap.get(var_name) {
        Some(val) => Ok(val.clone()),
        None => Err(RunTimeError::SyntaxError("Variable not found".to_string())),
    }
}

pub fn parse_var_assignment(
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

pub fn parse_func_application(
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

            let matrix = Matrix::zero(value.clone().pow(2) as usize, 1);
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

            if !vector.is_vector() || vector.size().0 != matrix.size().1 || !matrix.is_hermitian() {
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

            if (vec.is_ok()) {
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

pub fn execute_ast_node(
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
    let selections = HashMap::<String, (String, MemoryLocation, i32, i32)>::new();

    let mut memory = QuantumMemory {
        heap,
        measurements,
        selections,
    };

    // LOOP TROUGH AST AND RUN
    for node in ast {
        println!("{:?}", node);
        println!("{:?}", memory.heap);
        execute_ast_node(&node, &mut memory).unwrap();
    }

    Ok(memory.measurements)
}

#[cfg(test)]
mod tests {
    use crate::quantum_assembler_parser::parse;

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
