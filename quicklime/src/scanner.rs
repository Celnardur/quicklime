use crate::token::*;
use crate::Error;
use TokenType::*;

use std::error;
use std::vec::Vec;
use std::format;


pub fn scan(code: Vec<char>) -> Result<Vec<Token>, Box<dyn error::Error>> {
    let mut index = 0;
    let mut tokens: Vec<Token> = Vec::new();

    while let Some(res) = parse_token(&code, index) {
        // TODO: Handle this
        let (token, length) = res.unwrap();
        match token {
            Whitespace => (),
            LineComment(_) => (),
            MultiLineComment(_) => (),
            _ => tokens.push(Token {
                start: index,
                length,
                kind: token,
            }),
        }
        index += length;
    }
    Ok(tokens)
}

macro_rules! oct {
    ( $token:expr ) => {
        Ok(($token, 1))
    };
}

macro_rules! tct {
    ( $token:expr ) => {
        Ok(($token, 2))
    };
}

// None indicates end of file
pub fn parse_token(
    code: &Vec<char>,
    start_index: usize,
) -> Option<Result<(TokenType, usize), Error>> {
    if start_index >= code.len() {
        return None;
    }
    let code = &code[start_index..];

    // Identifiers and keywords
    if code[0].is_alphabetic() {
        let mut length = 1;
        while length < code.len() && (code[length].is_alphanumeric() || code[length] == '_') {
            length += 1;
        }

        let id_owned = code[..length].iter().collect::<String>();
        let id = id_owned.as_str();
        // a keyword is just a special identifier
        return Some(Ok((match id {
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
            _ => Identifier(id_owned),
        }, length)));
    }

    // check for whitespace
    if code[0].is_whitespace() {
        let mut length = 1;
        while code.get(length).is_some() && code[length].is_whitespace() {
            length += 1;
        }
        return Some(Ok((Whitespace, length)));
    }

    // check for number literals
    if code[0].is_ascii_digit() {
        let mut length = 1;
        while length < code.len() && code[length].is_ascii_digit() {
            length += 1;
        }
        // double literal
        return Some(if length < code.len() && code[length] == '.' {
            length += 1;
            while length < code.len() && code[length].is_ascii_digit() {
                length += 1;
            }

            let token: String = code[..length].iter().collect();
            match token.parse() {
                Ok(number) => Ok((Double(number), length)),
                Err(_) => Err(Error::simple_error(
                    &format!("Invalid Token: Cannot parse '{}' as a 64 bit floating point", token),
                    1, start_index, length,
                    "This number cannot fit in a f64", // NOTE: consider different message
                )),
            }
        } else {
            let token: String = code[..length].iter().collect();
            match token.parse() {
                Ok(number) => Ok((Integer(number), length)),
                Err(_) => Err(Error::simple_error(
                    &format!("Invalid Token: Cannot parse '{}' as an 128 bit integer", token),
                    2, start_index, length,
                    "This number cannot fit in an i128", // NOTE: consider different message
                )),
            }
        });
    }

    Some(match code[0] {
        '(' => oct!(LParen),
        ')' => oct!(RParen),
        '[' => oct!(LSquare),
        ']' => oct!(RSquare),
        '{' => oct!(LCurly),
        '}' => oct!(RCurly),
        '+' => oct!(Plus),
        '-' => oct!(Minus),
        '*' => oct!(Multiply),
        '/' => match code.get(1) {
            Some('/') => {
                let mut length = 2;
                while code.get(length) != Some(&'\n') {
                    length += 1;
                }
                Ok((
                    LineComment(code[2..length].iter().clone().collect()),
                    length,
                ))
            }
            Some('*') => {
                let mut length = 2;
                while code.get(length) != None
                    && !(code.get(length) == Some(&'*') && code.get(length + 1) == Some(&'/'))
                {
                    length += 1;
                }

                // reached end of file while reading line comment
                if code.get(length) == None {
                    Ok((
                        MultiLineComment(code[2..].iter().cloned().collect()),
                        length
                    ))

                } else {
                    Ok((
                        MultiLineComment(code[2..length].iter().cloned().collect()),
                        length + 2,
                    ))
                }
            }
            _ => oct!(Divide),
        },
        '%' => oct!(Modulus),
        '<' => match code.get(1) {
            Some('=') => tct!(LE),
            Some('<') => tct!(LeftShift),
            _ => oct!(LT),
        },
        '>' => match code.get(1) {
            Some('=') => tct!(GE),
            Some('>') => tct!(RightShift),
            _ => oct!(GT),
        },
        '&' => match code.get(1) {
            Some('&') => tct!(And),
            _ => oct!(BitwiseAnd),
        },
        '|' => match code.get(1) {
            Some('|') => tct!(Or),
            _ => oct!(BitwiseOr),
        },
        _ => Err(Error::simple_error(
            &format!("Invalid Token: '{}' is an illegal character", code[0]),
            0, start_index, 1,
            "This character is not used in Quicklime",
        )),
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_token_test() {
        assert_eq!(
            parse_token(&"42".chars().collect(), 0).unwrap().unwrap(),
            (Integer(42), 2)
        );

        assert_eq!(
            parse_token(&"asdf".chars().collect(), 0).unwrap().unwrap(),
            (Identifier("asdf".to_string()), 4)
        );
    }

    #[test]
    fn indent() {
        let code = "asdf hello world".chars().collect();
        assert_eq!(
            scan(code).unwrap(),
            [
                Token {
                    start: 0,
                    length: 4,
                    kind: Identifier("asdf".to_string()),
                },
                Token {
                    start: 5,
                    length: 5,
                    kind: Identifier("hello".to_string()),
                },
                Token {
                    start: 11,
                    length: 5,
                    kind: Identifier("world".to_string()),
                },
            ]
        );
    }

    #[test]
    fn keyword() {
        let code = "i64 enum function let return while if".chars().collect();
        assert_eq!(
            scan(code)
                .unwrap()
                .iter()
                .map(|t| t.kind.clone())
                .collect::<Vec<_>>(),
            [I64, Enum, Function, Let, Return, While, If,]
        )
    }

    #[test]
    fn numbers() {
        let code = "42 0 0.1 3.14".chars().collect();
        assert_eq!(
            scan(code)
                .unwrap()
                .iter()
                .map(|t| t.kind.clone())
                .collect::<Vec<_>>(),
            [Integer(42), Integer(0), Double(0.1), Double(3.14)]
        );

        let code = "1234 3.14".chars().collect();
        assert_eq!(
            scan(code).unwrap(),
            [
                Token {
                    start: 0,
                    length: 4,
                    kind: Integer(1234),
                },
                Token {
                    start: 5,
                    length: 4,
                    kind: Double(3.14),
                },
            ]
        );
    }

    #[test]
    fn whitespace() {
        let code = "for\n\nwhile   \t \n for \t for".chars().collect();
        assert_eq!(
            scan(code).unwrap(),
            [
                Token {
                    start: 0,
                    length: 3,
                    kind: For,
                },
                Token {
                    start: 5,
                    length: 5,
                    kind: While,
                },
                Token {
                    start: 17,
                    length: 3,
                    kind: For,
                },
                Token {
                    start: 23,
                    length: 3,
                    kind: For,
                },
            ]
        );
    }

    #[test]
    fn comments() {
        let code = "for // line comment\nwhile /* multiline \n comment */\n let /* the end".chars().collect();
        assert_eq!(
            scan(code).unwrap(),
            [
                Token {
                    start: 0,
                    length: 3,
                    kind: For,
                },
                Token {
                    start: 20,
                    length: 5,
                    kind: While,
                },
                Token {
                    start: 53,
                    length: 3,
                    kind: Let,
                },
            ]
        );
    }
}
