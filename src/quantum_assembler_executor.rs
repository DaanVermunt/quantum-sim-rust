use std::{collections::HashMap, error, fmt};

use crate::{
    c, mat,
    quantum_assembler_parser::{ASTNode, AST},
    Matrix,
    C,
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

pub fn parse_literal(v: &String) -> Result<LiteralValue, RunTimeError> {
    match &v[..] {
        "G_H" => Err(RunTimeError::NotImplemented),
        "G_R_2" => Err(RunTimeError::NotImplemented),
        "G_R_4" => Err(RunTimeError::NotImplemented),
        "G_I" => Err(RunTimeError::NotImplemented),
        "G_CNOT" => Err(RunTimeError::NotImplemented),
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
    measurements: &mut HashMap<String, Matrix>,
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
    measurements: &mut HashMap<String, Matrix>,
) -> Result<Option<LiteralValue>, RunTimeError> {
    let params = params
        .iter()
        .map(|p| execute_ast_node(p, heap, measurements).unwrap())
        .filter_map(|p| p)
        .collect::<Vec<LiteralValue>>();

    match &func[..] {
        "INITIALIZE" => {
            if params.len() != 1 {
                return Err(RunTimeError::SyntaxError(
                    "Invalid number of parameters".to_string(),
                ));
            }

            let value = match &params[0] {
                LiteralValue::Int(i) => i,
                _ => {
                    return Err(RunTimeError::SyntaxError(
                        "Invalid variable name".to_string(),
                    ))
                }
            };

            Ok(Some(LiteralValue::Matrix(Matrix::zero(*value as usize, 1))))
        }
        "INVERSE" => Err(RunTimeError::NotImplemented),
        "TENSOR" => Err(RunTimeError::NotImplemented),
        "CONCAT" => Err(RunTimeError::NotImplemented),
        "APPLY" => Err(RunTimeError::NotImplemented),
        "SELECT" => Err(RunTimeError::NotImplemented),
        "MEASURE" => {
            if params.len() != 2 {
                return Err(RunTimeError::SyntaxError(
                    "Invalid number of parameters".to_string(),
                ));
            }

            let matrix = match &params[0] {
                LiteralValue::Matrix(m) => m,
                _ => {
                    return Err(RunTimeError::SyntaxError(
                        "Invalid variable name".to_string(),
                    ))
                }
            };

            let var_name = match &params[1] {
                LiteralValue::String(i) => i.to_string(),
                _ => {
                    return Err(RunTimeError::SyntaxError(
                        "Invalid variable name".to_string(),
                    ))
                }
            };

            measurements.insert(var_name, matrix.clone());
            Ok(None)
        }
        _ => Err(RunTimeError::NotImplemented),
    }
}

pub fn execute_ast_node(
    ast_node: &ASTNode,
    heap: &mut HashMap<String, LiteralValue>,
    measurements: &mut HashMap<String, Matrix>,
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

pub fn execute_script(ast: AST) -> Result<HashMap<String, Matrix>, RunTimeError> {
    let mut heap = HashMap::<String, LiteralValue>::new();
    let mut measurements = HashMap::<String, Matrix>::new();

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
    fn test_executor() {
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
        assert_eq!(res.get("RES").unwrap().clone(), mat![c!(0); c!(0)]);
    }
}
