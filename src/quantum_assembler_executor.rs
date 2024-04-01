use std::{collections::HashMap, error, f64::consts::PI, fmt, vec};

use crate::{
    c, cnot, hadamard, mat, matrix, measure_vec, phase_shift,
    quantum_assembler_parser::{ASTNode, AST},
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

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Matrix(Matrix),
    Int(i32),
    String(String),
}

pub fn unwrap_matrix(lit: &LiteralValue) -> Result<&Matrix, RunTimeError> {
    match lit {
        LiteralValue::Matrix(m) => Ok(m),
        _ => Err(RunTimeError::SyntaxError("Invalid matrix".to_string())),
    }
}

pub fn unwrap_int(lit: &LiteralValue) -> Result<&i32, RunTimeError> {
    match lit {
        LiteralValue::Int(m) => Ok(m),
        _ => Err(RunTimeError::SyntaxError("Invalid matrix".to_string())),
    }
}

pub fn unwrap_string(lit: &LiteralValue) -> Result<&String, RunTimeError> {
    match lit {
        LiteralValue::String(m) => Ok(m),
        _ => Err(RunTimeError::SyntaxError("Invalid matrix".to_string())),
    }
}

pub fn validate_param_len(params: &Vec<LiteralValue>, expected: usize) -> Result<(), RunTimeError> {
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
                Ok(LiteralValue::Int(v.parse::<i32>().unwrap()))
            } else {
                Ok(LiteralValue::String(v.clone()))
            }
        }
    }
}

pub fn parse_identifier(
    var_name: &String,
    heap: &HashMap<String, LiteralValue>,
) -> Result<LiteralValue, RunTimeError> {
    match heap.get(var_name) {
        Some(val) => Ok(val.clone()),
        None => Err(RunTimeError::SyntaxError("Variable not found".to_string())),
    }
}

pub fn parse_var_assignment(
    var_name: &String,
    val: &ASTNode,
    heap: &mut HashMap<String, LiteralValue>,
    measurements: &mut HashMap<String, (Matrix, String)>,
) -> Result<Option<LiteralValue>, RunTimeError> {
    let val = execute_ast_node(val, heap, measurements).unwrap();
    match val {
        Some(val) => {
            heap.insert(var_name.clone(), val);
            Ok(None)
        }
        None => Err(RunTimeError::SyntaxError("Variable not found".to_string())),
    }
}

pub fn parse_func_application(
    func: &String,
    params: &Vec<ASTNode>,
    heap: &mut HashMap<String, LiteralValue>,
    measurements: &mut HashMap<String, (Matrix, String)>,
) -> Result<Option<LiteralValue>, RunTimeError> {
    let params = params
        .iter()
        .map(|p| execute_ast_node(p, heap, measurements).unwrap())
        .filter_map(|p| p)
        .collect::<Vec<LiteralValue>>();

    match &func[..] {
        "INITIALIZE" => {
            validate_param_len(&params, 1).unwrap();

            let value = unwrap_int(&params[0]).unwrap();

            let matrix = Matrix::zero(value.clone().pow(2) as usize, 1);
            Ok(Some(LiteralValue::Matrix(matrix.set(0, 0, c!(1)))))
        }
        "INVERSE" => {
            validate_param_len(&params, 1).unwrap();

            let matrix = unwrap_matrix(&params[0]).unwrap();

            if !matrix.is_hermitian() {
                return Err(RunTimeError::SyntaxError(
                    "Input invalid for INVERSE, should be a hermetian matrix".to_string(),
                ));
            }

            Ok(Some(LiteralValue::Matrix(matrix.adjoint())))
        }
        "TENSOR" => {
            validate_param_len(&params, 2).unwrap();

            let matrix1 = unwrap_matrix(&params[0]).unwrap();
            let matrix2 = unwrap_matrix(&params[1]).unwrap();

            Ok(Some(LiteralValue::Matrix(matrix1.tensor(matrix2))))
        }
        "CONCAT" => {
            validate_param_len(&params, 2).unwrap();

            let matrix1 = unwrap_matrix(&params[0]).unwrap();
            let matrix2 = unwrap_matrix(&params[1]).unwrap();

            if matrix1.size() != matrix2.size() {
                return Err(RunTimeError::SyntaxError(
                    "Matrix sizes should be equal to CONCAT".to_string(),
                ));
            }

            Ok(Some(LiteralValue::Matrix(matrix1 * matrix2)))
        }
        "APPLY" => {
            validate_param_len(&params, 2).unwrap();

            let matrix = unwrap_matrix(&params[0]).unwrap();
            let vector = unwrap_matrix(&params[1]).unwrap();

            if !vector.is_vector() || vector.size().0 != matrix.size().1 || !matrix.is_hermitian() {
                return Err(RunTimeError::SyntaxError(
                    "Input invalid for APPLY, first arg should be a hermetian matrix & the second arg should be vector with equal columns".to_string(),
                ));
            }

            Ok(Some(LiteralValue::Matrix(matrix * vector)))
        }
        "SELECT" => {
            validate_param_len(&params, 3).unwrap();

            let matrix = unwrap_matrix(&params[0]).unwrap();
            let start = unwrap_int(&params[1]).unwrap();
            let end = unwrap_int(&params[2]).unwrap();

            if start > end || (*end as usize) > matrix.size().0 {
                return Err(RunTimeError::SyntaxError(
                    "Invalid range for SELECT".to_string(),
                ));
            }

            // TODO: Implement select
            Err(RunTimeError::NotImplemented)
        }
        "MEASURE" => {
            validate_param_len(&params, 2).unwrap();

            let vec = unwrap_matrix(&params[0]).unwrap();
            let var_name = unwrap_string(&params[1]).unwrap().to_string();

            if !vec.is_vector() {
                return Err(RunTimeError::SyntaxError(
                    "Invalid input for MEASURE, should be a vector".to_string(),
                ));
            }

            measurements.insert(var_name, (vec.clone(), measure_vec(vec)));
            Ok(None)
        }
        _ => Err(RunTimeError::NotImplemented),
    }
}

pub fn execute_ast_node(
    ast_node: &ASTNode,
    heap: &mut HashMap<String, LiteralValue>,
    measurements: &mut HashMap<String, (Matrix, String)>,
) -> Result<Option<LiteralValue>, RunTimeError> {
    match ast_node {
        ASTNode::Literal(val) => Ok(Some(parse_literal(val).unwrap())),
        ASTNode::Identifier(var_name) => Ok(Some(parse_identifier(var_name, heap).unwrap())),
        ASTNode::VariableAssignment(var_name, val) => {
            parse_var_assignment(var_name, &*val, heap, measurements).unwrap();
            Ok(None)
        }
        ASTNode::FunctionApplication(func, params) => {
            parse_func_application(func, params, heap, measurements)
        }
    }
}

pub fn execute_script(ast: AST) -> Result<HashMap<String, (Matrix, String)>, RunTimeError> {
    let mut heap = HashMap::<String, LiteralValue>::new();
    let mut measurements = HashMap::<String, (Matrix, String)>::new();

    // LOOP TROUGH AST AND RUN
    for node in ast {
        println!("{:?}", node);
        println!("{:?}", heap);
        execute_ast_node(&node, &mut heap, &mut measurements).unwrap();
    }

    Ok(measurements)
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
        MEASURE R 'RES'
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
        MEASURE R 'RES'
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
            MEASURE S1 'RES'
            APPLY CNOT R
            MEASURE R 'RES'
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
}
