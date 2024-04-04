use std::{error, fmt, rc::Rc};

use crate::quantum_assembler_lexer::{tokenize, Token, TokenType};

#[derive(Debug, Clone, PartialEq)]
pub enum MemoryLocation {
    Heap,
    Measurement,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTNode {
    Literal(String),
    Identifier(String),
    VariableAssignment(String, MemoryLocation, Rc<ASTNode>),

    FunctionApplication(String, Vec<ASTNode>),
}

pub type AST = Vec<ASTNode>;

#[derive(Debug)]
pub enum ParseError {
    SyntaxError(String), // TOO GENERIC
    NotImplemented,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::SyntaxError(mess) => write!(f, "Syntax error: {}", mess),
            ParseError::NotImplemented => write!(f, "Not implemented"),
        }
    }
}

impl error::Error for ParseError {
    fn description(&self) -> &str {
        match self {
            ParseError::SyntaxError(mess) => "Syntax error in code",
            ParseError::NotImplemented => "Not implemented",
        }
    }
}

pub fn parse_param(param: &Token) -> Result<ASTNode, ParseError> {
    match param.token_type {
        TokenType::Literal => Ok(ASTNode::Literal(param.value.clone())),
        TokenType::Prefabs => Ok(ASTNode::Literal(param.value.clone())),
        TokenType::Identifier => Ok(ASTNode::Identifier(param.value.clone())),
        _ => Err(ParseError::SyntaxError(format!(
            "Invalid paramater {} - {:?}",
            param.value, param.token_type
        ))),
    }
}

pub fn parse_dual_token_group(
    action: &Token,
    param0: &Token,
    param1: &Token,
) -> Result<ASTNode, ParseError> {
    match action.value.as_str() {
        "APPLY" => Ok(ASTNode::VariableAssignment(
            param1.value.clone(),
            MemoryLocation::Heap,
            Rc::new(ASTNode::FunctionApplication(
                action.value.clone(),
                vec![parse_param(param0).unwrap(), parse_param(param1).unwrap()],
            )),
        )),
        "INITIALIZE" => Ok(ASTNode::VariableAssignment(
            param0.value.clone(),
            MemoryLocation::Heap,
            Rc::new(ASTNode::FunctionApplication(
                action.value.clone(),
                vec![parse_param(param1).unwrap()],
            )),
        )),
        "MEASURE" => Ok(ASTNode::VariableAssignment(
            param1.value.clone(),
            MemoryLocation::Measurement,
            Rc::new(ASTNode::FunctionApplication(
                action.value.clone(),
                vec![parse_param(param0).unwrap()],
            )),
        )),
        _ => Err(ParseError::SyntaxError(format!(
            "Invalid dual action {} - {:?}",
            action.value, action.token_type
        ))),
    }
}

pub fn parse_quat_token_group(
    action: &Token,
    param0: &Token,
    param1: &Token,
    param2: &Token,
    param3: &Token,
) -> Result<ASTNode, ParseError> {
    match action.value.as_str() {
        "SELECT" => Ok(ASTNode::VariableAssignment(
            param0.value.clone(),
            MemoryLocation::Heap,
            Rc::new(ASTNode::FunctionApplication(
                action.value.clone(),
                vec![
                    parse_param(param1).unwrap(),
                    parse_param(param2).unwrap(),
                    parse_param(param3).unwrap(),
                ],
            )),
        )),
        _ => Err(ParseError::SyntaxError(format!(
            "Invalid quat action {} - {:?}",
            action.value, action.token_type
        ))),
    }
}

pub fn parse_ass_single_token_group(
    action: &Token,
    ass: &Token,
    param1: &Token,
) -> Result<ASTNode, ParseError> {
    match action.value.as_str() {
        "INVERSE" => Ok(ASTNode::VariableAssignment(
            ass.value.clone(),
            MemoryLocation::Heap,
            Rc::new(ASTNode::FunctionApplication(
                action.value.clone(),
                vec![parse_param(param1).unwrap()],
            )),
        )),
        _ => Err(ParseError::SyntaxError(format!(
            "Invalid single assign action {} - {:?}",
            action.value, action.token_type
        ))),
    }
}

pub fn parse_ass_dual_token_group(
    action: &Token,
    ass: &Token,
    param1: &Token,
    param2: &Token,
) -> Result<ASTNode, ParseError> {
    match action.value.as_str() {
        "TENSOR" | "CONCAT" => Ok(ASTNode::VariableAssignment(
            ass.value.clone(),
            MemoryLocation::Heap,
            Rc::new(ASTNode::FunctionApplication(
                action.value.clone(),
                vec![parse_param(param1).unwrap(), parse_param(param2).unwrap()],
            )),
        )),
        _ => Err(ParseError::SyntaxError(format!(
            "Invalid dual assign action {} - {:?}",
            action.value, action.token_type
        ))),
    }
}

pub fn parse_vector_init(ass: &Token, params: &Vec<Token>) -> Result<ASTNode, ParseError> {
    let res = ASTNode::VariableAssignment(
        ass.value.clone(),
        MemoryLocation::Heap,
        Rc::new(ASTNode::FunctionApplication(
            "INITIALIZE".to_string(),
            vec![ASTNode::FunctionApplication(
                "VECTOR".to_string(),
                params
                    .clone()
                    .iter()
                    .map(|p| parse_param(&p).unwrap())
                    .collect::<Vec<ASTNode>>(),
            )],
        )),
    );

    Ok(res)
}

pub fn parse_token_group(inp: Vec<Token>) -> Result<ASTNode, ParseError> {
    let type_vec: Vec<TokenType> = inp.iter().map(|t| t.token_type).collect();
    match type_vec.as_slice() {
        [TokenType::Action, _, _] => parse_dual_token_group(&inp[0], &inp[1], &inp[2]), // e.g APPLY U R
        [TokenType::Action, TokenType::Identifier, TokenType::OpenBracket, .., TokenType::CloseBracket] => {
            parse_vector_init(&inp[1], &inp[3..(inp.len() - 1)].to_vec())
        } // e.g INITIALIZE R [1, 2, 3]
        [TokenType::Action, _, _, _, _] => {
            parse_quat_token_group(&inp[0], &inp[1], &inp[2], &inp[3], &inp[4])
        } // e.g SELECT S1 R1 2 3
        [TokenType::Identifier, TokenType::Action, _] => {
            parse_ass_single_token_group(&inp[1], &inp[0], &inp[2])
        } // e.g U2 INVERSE U1
        [TokenType::Identifier, TokenType::Action, _, _] => {
            parse_ass_dual_token_group(&inp[1], &inp[0], &inp[2], &inp[3])
        } // e.g. R2 TENSOR U1 U2
        _ => Err(ParseError::SyntaxError(format!(
            "Invalid action pattern: {}",
            inp.iter()
                .map(|i| i.value.clone())
                .collect::<Vec<String>>()
                .join(" ")
        ))),
    }
}

pub fn parse(inp: String) -> Result<Vec<ASTNode>, ParseError> {
    let tokens = tokenize(inp);

    // TODO SPLIT BY NEWLINE
    // MATCH EXPRESSION AND PARSE
    let groups: Vec<&[Token]> = tokens
        .split(|t| t.token_type == TokenType::NewLine)
        .filter(|g| g.len() > 0)
        .collect();

    let res: Vec<ASTNode> = groups
        .into_iter()
        .map(|g| parse_token_group(g.to_vec()).unwrap())
        .collect();
    Ok(res)
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn test_parse_basic() {
        let input = "INITIALIZE R 2
        U TENSOR G_H G_H
        APPLY U R
        MEASURE R RES"
            .to_string();
        let res = parse(input);

        assert!(res.is_ok());
        assert_eq!(
            res.unwrap(),
            vec![
                ASTNode::VariableAssignment(
                    "R".to_string(),
                    MemoryLocation::Heap,
                    Rc::new(ASTNode::FunctionApplication(
                        "INITIALIZE".to_string(),
                        vec![ASTNode::Literal("2".to_string())]
                    ))
                ),
                ASTNode::VariableAssignment(
                    "U".to_string(),
                    MemoryLocation::Heap,
                    Rc::new(ASTNode::FunctionApplication(
                        "TENSOR".to_string(),
                        vec![
                            ASTNode::Literal("G_H".to_string()),
                            ASTNode::Literal("G_H".to_string())
                        ]
                    ))
                ),
                ASTNode::VariableAssignment(
                    "R".to_string(),
                    MemoryLocation::Heap,
                    Rc::new(ASTNode::FunctionApplication(
                        "APPLY".to_string(),
                        vec![
                            ASTNode::Identifier("U".to_string()),
                            ASTNode::Identifier("R".to_string())
                        ]
                    ))
                ),
                ASTNode::VariableAssignment(
                    "RES".to_string(),
                    MemoryLocation::Measurement,
                    Rc::new(ASTNode::FunctionApplication(
                        "MEASURE".to_string(),
                        vec![ASTNode::Identifier("R".to_string())]
                    )),
                )
            ]
        );
    }

    #[test]
    fn test_parse_init_vec() {
        let input = "INITIALIZE R [1 2 3]
        INITIALIZE R []
        INITIALIZE R [1]"
            .to_string();
        let res = parse(input);

        assert!(res.is_ok());
        assert_eq!(
            res.unwrap(),
            vec![
                ASTNode::VariableAssignment(
                    "R".to_string(),
                    MemoryLocation::Heap,
                    Rc::new(ASTNode::FunctionApplication(
                        "INITIALIZE".to_string(),
                        vec![ASTNode::FunctionApplication(
                            "VECTOR".to_string(),
                            vec![
                                ASTNode::Literal("1".to_string()),
                                ASTNode::Literal("2".to_string()),
                                ASTNode::Literal("3".to_string())
                            ]
                        )]
                    ))
                ),
                ASTNode::VariableAssignment(
                    "R".to_string(),
                    MemoryLocation::Heap,
                    Rc::new(ASTNode::FunctionApplication(
                        "INITIALIZE".to_string(),
                        vec![ASTNode::FunctionApplication("VECTOR".to_string(), vec![])]
                    ))
                ),
                ASTNode::VariableAssignment(
                    "R".to_string(),
                    MemoryLocation::Heap,
                    Rc::new(ASTNode::FunctionApplication(
                        "INITIALIZE".to_string(),
                        vec![ASTNode::FunctionApplication(
                            "VECTOR".to_string(),
                            vec![ASTNode::Literal("1".to_string()),]
                        )]
                    ))
                ),
            ]
        );
    }

    #[test]
    fn test_select() {
        let input = "SELECT S1 R1 2 3
        SELECT S2 R2 4 5"
            .to_string();
        let res = parse(input);

        assert!(res.is_ok());
        assert_eq!(
            res.unwrap(),
            vec![
                ASTNode::VariableAssignment(
                    "S1".to_string(),
                    MemoryLocation::Heap,
                    Rc::new(ASTNode::FunctionApplication(
                        "SELECT".to_string(),
                        vec![
                            ASTNode::Identifier("R1".to_string()),
                            ASTNode::Literal("2".to_string()),
                            ASTNode::Literal("3".to_string())
                        ]
                    ))
                ),
                ASTNode::VariableAssignment(
                    "S2".to_string(),
                    MemoryLocation::Heap,
                    Rc::new(ASTNode::FunctionApplication(
                        "SELECT".to_string(),
                        vec![
                            ASTNode::Identifier("R2".to_string()),
                            ASTNode::Literal("4".to_string()),
                            ASTNode::Literal("5".to_string())
                        ]
                    ))
                ),
            ]
        );
    }

    #[test]
    fn test_empty_lines() {
        let input = "

        INITIALIZE R 2


        MEASURE R RES

        "
        .to_string();
        let res = parse(input);

        assert!(res.is_ok());

        let res = res.unwrap();
        assert_eq!(
            res,
            vec![
                ASTNode::VariableAssignment(
                    "R".to_string(),
                    MemoryLocation::Heap,
                    Rc::new(ASTNode::FunctionApplication(
                        "INITIALIZE".to_string(),
                        vec![ASTNode::Literal("2".to_string())]
                    ))
                ),
                ASTNode::VariableAssignment(
                    "RES".to_string(),
                    MemoryLocation::Measurement,
                    Rc::new(ASTNode::FunctionApplication(
                        "MEASURE".to_string(),
                        vec![ASTNode::Identifier("R".to_string())]
                    )),
                ),
            ]
        );
    }
}
