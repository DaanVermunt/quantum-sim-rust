#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TokenType {
    Action,
    Prefabs,
    Identifier,

    Literal,

    OpenBracket,
    CloseBracket,

    NewLine,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
}

fn match_token_type(token: &String) -> TokenType {
    match token.as_str() {
        "INITIALIZE" | "MEASURE" | "SELECT" | "APPLY" | "CONCAT" | "TENSOR" | "INVERSE" => {
            TokenType::Action
        }
        "G_H" | "G_R_2" | "G_R_4" | "G_I_2" | "G_I_4" | "G_I_8" | "G_CNOT" => TokenType::Prefabs, // TODO: MAKE _2 _4 params
        _ => {
            if token.parse::<i32>().is_ok() {
                TokenType::Literal
            } else {
                TokenType::Identifier
            }
        }
    }
}

fn push_current_token(tokens: &mut Vec<Token>, current_token: &mut String) {
    if current_token.len() > 0 {
        let token_type = match_token_type(&current_token);

        tokens.push(Token {
            token_type: token_type,
            value: current_token.replace("'", "").clone(),
        });

        current_token.clear();
    }
}

pub fn tokenize(inp: String) -> Vec<Token> {
    let mut tokens = Vec::new();

    let mut current_token = String::new();

    for c in inp.chars() {
        match c {
            ' ' => {
                push_current_token(&mut tokens, &mut current_token);
            }
            '\n' => {
                push_current_token(&mut tokens, &mut current_token);
                tokens.push(Token {
                    token_type: TokenType::NewLine,
                    value: "\n".to_string(),
                });
            }
            '[' => {
                push_current_token(&mut tokens, &mut current_token);

                tokens.push(Token {
                    token_type: TokenType::OpenBracket,
                    value: "[".to_string(),
                });
            }
            ']' => {
                push_current_token(&mut tokens, &mut current_token);

                tokens.push(Token {
                    token_type: TokenType::CloseBracket,
                    value: "]".to_string(),
                });
            }
            _ => {
                current_token.push(c);
            }
        }
    }

    push_current_token(&mut tokens, &mut current_token);

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_lexer() {
        let inp = "INITIALIZE R 2
        MEASURE R 'RES'";

        let tokens = tokenize(inp.to_string());
        assert_eq!(tokens.len(), 7);
        assert_eq!(
            tokens,
            vec![
                Token {
                    token_type: TokenType::Action,
                    value: "INITIALIZE".to_string()
                },
                Token {
                    token_type: TokenType::Identifier,
                    value: "R".to_string()
                },
                Token {
                    token_type: TokenType::Literal,
                    value: "2".to_string()
                },
                Token {
                    token_type: TokenType::NewLine,
                    value: "\n".to_string()
                },
                Token {
                    token_type: TokenType::Action,
                    value: "MEASURE".to_string()
                },
                Token {
                    token_type: TokenType::Identifier,
                    value: "R".to_string()
                },
                Token {
                    token_type: TokenType::Identifier,
                    value: "RES".to_string()
                },
            ]
        )
    }

    #[test]
    fn test_literals() {
        let inp = "INITIALIZE 2 3";
        let tokens = tokenize(inp.to_string());
        assert_eq!(tokens.len(), 3);
        assert_eq!(
            tokens[1],
            Token {
                token_type: TokenType::Literal,
                value: "2".to_string()
            }
        );
        assert_eq!(
            tokens[2],
            Token {
                token_type: TokenType::Literal,
                value: "3".to_string()
            }
        );
    }

    #[test]
    fn test_bit_array() {
        let inp = "INITIALIZE R2 [0 0 ]";
        let tokens = tokenize(inp.to_string());
        assert_eq!(tokens.len(), 6);
        assert_eq!(
            tokens[2],
            Token {
                token_type: TokenType::OpenBracket,
                value: "[".to_string()
            }
        );
        assert_eq!(
            tokens[3],
            Token {
                token_type: TokenType::Literal,
                value: "0".to_string()
            }
        );
        assert_eq!(
            tokens[5],
            Token {
                token_type: TokenType::CloseBracket,
                value: "]".to_string()
            }
        );
    }
}
