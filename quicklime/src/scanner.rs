use crate::token::*;
use TokenType::*;
use std::vec::Vec;
use std::error;

pub fn scan(code: Vec<char>) -> Result<Vec<Token>, Box<dyn error::Error>> {
    let mut on = Pos {
        line: 0,
        col: 0,
    };
    let mut index = 0;
    let mut tokens: Vec<Token> = Vec::new();

    while let Some((token, pos, length)) = parse_token(&code, index)? {
        match token {
            Whitespace => (),
            _ => tokens.push(Token { start: on.clone(), end: on.add(&pos), kind: token})
        }
        on = on.add(&pos);
        index += length;
    }
    Ok(tokens)
}

pub fn parse_token(code: &Vec<char>, start_index: usize)
    -> Result<Option<(TokenType, Pos, usize)>, Box<dyn error::Error>> {
    if start_index >= code.len() {
        return Ok(None);
    }
    let code = &code[start_index..];

    // Identifiers and keywords
    if code[0].is_alphabetic() {
        let mut length = 1;
        while length < code.len() && code[length].is_alphanumeric(){
            length += 1;
        }

        let pos = Pos {line: 0, col: length};
        let id = code[..length].iter().collect::<String>();
        let id = id.as_str();
        // a keyword is just a special identifier
        let token = match id {
            "i64" => I64,
            "u64" => U64,
            "u8" => U8,
            "f64" => F64,
            "bool" => Bool,
            "char" => Char,
            "type" => Type,
            "enum" => Enum,
            "let" => Let,
            "mut" => Mut,
            "function" => Function,
            "return" => Return,
            "yield" => Yield,
            "while" => While,
            "for" => For,
            "if" => If,
            "else" => Else,
            "match" => Match,
            _ => return Ok(Some((Identifier(id.to_owned()), pos, length))),
        };
        return Ok(Some((token, pos, length)));
    }

    // check for whitespace
    if code[0].is_whitespace() {
        let mut length = 1;
        while length < code.len() && code[length].is_whitespace() {
            length += 1;
        }
        return Ok(Some((Whitespace, Pos {col: length, line: 0}, length)))
    }

    // check for number literals
    if code[0].is_ascii_digit() {
        let mut length = 1;
        while length < code.len() && code[length].is_ascii_digit() {
            length += 1;
        }
        // double literal
        let token = if length < code.len() && code[length] == '.' {
            length += 1;
            while length < code.len() && code[length].is_ascii_digit() {
                length += 1;
            }
            // TODO: handle bad parses
            Double(code[..length].iter().collect::<String>().parse()?)
        } else {
            Integer(code[..length].iter().collect::<String>().parse()?)
        };

        return Ok(Some((token, Pos {col: length, line: 0}, length)))
    }

    match code[0] {
        '(' => Ok(Some((LParen, Pos {line: 0, col: 1}, 1))),
        ')' => Ok(Some((RParen, Pos {line: 0, col: 1}, 1))),
        '[' => Ok(Some((LSquare, Pos {line: 0, col: 1}, 1))),
        ']' => Ok(Some((RSquare, Pos {line: 0, col: 1}, 1))),
        '{' => Ok(Some((LCurly, Pos {line: 0, col: 1}, 1))),
        '}' => Ok(Some((RCurly, Pos {line: 0, col: 1}, 1))),
        '+' => Ok(Some((Plus, Pos {line: 0, col: 1}, 1))),
        '-' => Ok(Some((Minus, Pos {line: 0, col: 1}, 1))),
        '*' => Ok(Some((Multiply, Pos {line: 0, col: 1}, 1))),
        '/' => match code.get(1) {
            Some('/') => {
                let mut length = 2;
                while code.get(length) != '\n' {
                    length += 1;
                }
                Ok(Some((
                    LineComment(code[2..length].iter().clone().collect()),
                    Pos {line: 0, col: length},
                    length
                )))
            },
            Some('*') => {
                let mut length = 2;
                let mut col = 2;
                let mut line = 0;
                while code.get(length) != None &&
                    !(code.get(length) == Some('*') && code.get(length + 1) == Some('/')) {
                    if code[length] == '\n' {
                        line += 1;
                        col = 0;
                    } else {
                        col += 1;
                    }
                    length += 1;
                }
                Ok(Some((
                    MultiLineComment(code[2..length].iter().cloned().collect()),
                    Pos { line, col: col + 2},
                    length + 2,
                )))
            },
            _ => Ok(Some((Divide, Pos {line: 0, col: 1}, 1))),
        }
        '%' => Ok(Some((Modulus, Pos {line: 0, col: 1}, 1))),
        '<' => match code.get(1) {
            Some('=') => Ok(Some((LE, Pos {line: 0, col: 2}, 2))),
            Some('<') => Ok(Some((LeftShift, Pos {line: 0, col:2 }, 2))),
            _ => Ok(Some((LT, Pos {line: 0, col: 1}, 1))),
        },
        '>' => match code.get(1) {
            Some('=') => Ok(Some((GE, Pos {line: 0, col: 2}, 2))),
            Some('>') => Ok(Some((RightShift, Pos {line: 0, col: 2}, 2))),
            _ => Ok(Some((GT, Pos {line: 0, col: 1}, 1))),
        },
        '&' => match code.get(1) {
            Some('&') => Ok(Some((And, Pos {line: 0, col: 2}, 2))),
            _ => Ok(Some((BitwiseAnd, Pos {line: 0, col: 1}, 1))),
        },
        '|' => match code.get(1) {
            Some('|') => Ok(Some((Or, Pos {line: 0, col: 2}, 2))),
            _ => Ok(Some((BitwiseOr, Pos {line: 0, col: 1}, 1))),
        },
        _ => Ok(None),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_token_test() {
        assert_eq!(
            parse_token(&"42".chars().collect(), 0).unwrap().unwrap(),
            (Integer(42), Pos {col: 2, line: 0}, 2)
        );

        assert_eq!(
            parse_token(&"asdf".chars().collect(), 0).unwrap().unwrap(),
            (Identifier("asdf".to_string()), Pos {col: 4, line: 0}, 4)
        );
    }

    #[test]
    fn indent() {
        let code = "asdf hello world".chars().collect();
        assert_eq!(
            scan(code).unwrap(),
            [
                Token {
                    start: Pos {line: 0, col: 0},
                    end: Pos {line: 0, col: 4},
                    kind: Identifier("asdf".to_string()),
                },
                Token {
                    start: Pos {line: 0, col: 5},
                    end: Pos {line: 0, col: 10},
                    kind: Identifier("hello".to_string()),
                },
                Token {
                    start: Pos {line: 0, col: 11},
                    end: Pos {line: 0, col: 16},
                    kind: Identifier("world".to_string()),
                },
            ]
        );
    }

    #[test]
    fn keyword() {
        let code = "i64 enum function let return while if".chars().collect();
        assert_eq!(
            scan(code).unwrap().iter().map(|t| t.kind.clone()).collect::<Vec<_>>(),
            [ I64, Enum, Function, Let, Return, While, If,]
        )
    }

    #[test]
    fn numbers() {
        let code = "42 0 0.1 3.14".chars().collect();
        assert_eq!(
            scan(code).unwrap().iter().map(|t| t.kind.clone()).collect::<Vec<_>>(),
            [ Integer(42), Integer(0), Double(0.1), Double(3.14)]
        );

        let code = "1234 3.14".chars().collect();
        assert_eq!(
            scan(code).unwrap(),
            [
                Token {
                    start: Pos { line: 0, col: 0 },
                    end: Pos { line: 0, col: 4 },
                    kind: Integer(1234),
                },
                Token {
                    start: Pos { line: 0, col: 5 },
                    end: Pos { line: 0, col: 9 },
                    kind: Double(3.14),
                },
            ]
        );
    }


}