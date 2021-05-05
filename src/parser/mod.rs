use crate::lexer::*;
use std::collections::HashMap;
use std::result;

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    Array(Box<[JsonValue]>),
    Object(HashMap<String, JsonValue>),
    String(String),
    Number(f64),
    True,
    False,
    Null,
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    InvalidKeyword,
    InvalidToken,
    Unknown,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    Lexer(LexerError),
    Parser(ParseError),
}

pub struct Parser {
    lexer: Lexer,
}

type Result = result::Result<JsonValue, Error>;

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self { lexer }
    }

    pub fn parse(&mut self) -> Result {
        if self.lexer.is_eot() {
            return Ok(JsonValue::Null);
        }

        match self.lexer.get_next_token() {
            Ok(t) => self.parse_value(t),
            Err(e) => Err(Error::Lexer(e)),
        }
    }

    fn parse_value(&mut self, token: Token) -> Result {
        match token {
            Token::String(s) => Ok(JsonValue::String(s)),
            Token::Number(f) => Ok(JsonValue::Number(f)),
            Token::LeftBracket => self.parse_array(),
            Token::LeftBrace => self.parse_object(),
            Token::Keyword(k) => match k.as_str() {
                "true" => Ok(JsonValue::True),
                "false" => Ok(JsonValue::False),
                "null" => Ok(JsonValue::Null),
                _ => Err(Error::Parser(ParseError::InvalidKeyword)),
            },
            _ => Err(Error::Parser(ParseError::Unknown)),
        }
    }

    fn parse_array(&mut self) -> Result {
        enum State {
            Default,
            Value,
            Comma,
        }

        let mut state = State::Default;
        let mut array: Vec<JsonValue> = vec![];

        loop {
            let token = match self.lexer.get_next_token() {
                Ok(t) => t,
                Err(e) => return Err(Error::Lexer(e)),
            };

            match state {
                State::Default => match token {
                    Token::RightBracket => return Ok(JsonValue::Array(array.into_boxed_slice())),
                    _ => {
                        array.push(self.parse_value(token)?);
                        state = State::Value;
                    }
                },
                State::Value => match token {
                    Token::RightBracket => return Ok(JsonValue::Array(array.into_boxed_slice())),
                    Token::Comma => state = State::Comma,
                    _ => return Err(Error::Parser(ParseError::InvalidToken)),
                },
                State::Comma => {
                    array.push(self.parse_value(token)?);
                    state = State::Value;
                }
            }
        }
    }

    fn parse_object(&mut self) -> Result {
        enum State {
            Default,
            Value,
            Comma,
            Colon,
            Key,
        }

        let mut state = State::Default;
        let mut key = String::new();
        let mut map: HashMap<String, JsonValue> = HashMap::new();

        loop {
            let token = match self.lexer.get_next_token() {
                Ok(t) => t,
                Err(e) => return Err(Error::Lexer(e)),
            };

            match state {
                State::Default => match token {
                    Token::RightBrace => return Ok(JsonValue::Object(map)),
                    Token::String(s) => {
                        key = s;
                        state = State::Key;
                    }
                    _ => return Err(Error::Parser(ParseError::InvalidToken)),
                },
                State::Key => match token {
                    Token::Colon => state = State::Colon,
                    _ => return Err(Error::Parser(ParseError::InvalidToken)),
                },
                State::Colon => {
                    if key.is_empty() {
                        return Err(Error::Parser(ParseError::InvalidToken));
                    }

                    map.insert(key.clone(), self.parse_value(token)?);
                    state = State::Value;
                }
                State::Value => match token {
                    Token::RightBrace => return Ok(JsonValue::Object(map)),
                    Token::Comma => state = State::Comma,
                    _ => return Err(Error::Parser(ParseError::InvalidToken)),
                },
                State::Comma => match token {
                    Token::String(s) => {
                        key = s;
                        state = State::Key;
                    }
                    _ => return Err(Error::Parser(ParseError::InvalidToken)),
                },
            }
        }
    }
}

#[test]
fn test_parse() {
    let lexer = Lexer::new(String::from("null"));
    let mut parser = Parser::new(lexer);
    let json = parser.parse();

    assert_eq!(json, Ok(JsonValue::Null));
}

#[test]
fn test_parse_array() {
    let lexer = Lexer::new(String::from("[1, \"abc\",[true,[]]]"));
    let mut parser = Parser::new(lexer);
    let json = parser.parse();

    assert_eq!(
        json,
        Ok(JsonValue::Array(
            vec![
                JsonValue::Number(1.0),
                JsonValue::String(String::from("abc")),
                JsonValue::Array(
                    vec![JsonValue::True, JsonValue::Array(vec![].into_boxed_slice())]
                        .into_boxed_slice()
                )
            ]
            .into_boxed_slice()
        ))
    );
}

#[test]
fn test_parse_object() {
    let lexer = Lexer::new(String::from(
        r#"{"n":1, "s": "abc", "a":[1,2], "o":{"k1":"hi"}}"#,
    ));
    let mut parser = Parser::new(lexer);
    let json = parser.parse();

    let mut map: HashMap<String, JsonValue> = HashMap::new();
    map.insert(String::from("n"), JsonValue::Number(1.0));
    map.insert(String::from("s"), JsonValue::String(String::from("abc")));
    map.insert(
        String::from("a"),
        JsonValue::Array(vec![JsonValue::Number(1.0), JsonValue::Number(2.0)].into_boxed_slice()),
    );

    let mut map1: HashMap<String, JsonValue> = HashMap::new();
    map1.insert(String::from("k1"), JsonValue::String(String::from("hi")));
    map.insert(String::from("o"), JsonValue::Object(map1));

    assert_eq!(json, Ok(JsonValue::Object(map)));
}
