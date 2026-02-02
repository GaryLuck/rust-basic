/// Lexer for Tiny BASIC - tokenizes source code
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(i32),
    Ident(char),
    String(String),
    // Keywords
    Print,
    Let,
    Goto,
    If,
    Then,
    End,
    Dim,
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Equals,
    LessThan,
    GreaterThan,
    LessEq,
    GreaterEq,
    NotEquals,
    // Punctuation
    LeftParen,
    RightParen,
    Comma,
}

#[derive(Debug)]
pub struct LexerError {
    pub message: String,
    pub position: usize,
}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at position {}", self.message, self.position)
    }
}

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    position: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.chars().peekable(),
            position: 0,
        }
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.input.next();
        if c.is_some() {
            self.position += 1;
        }
        c
    }

    fn peek(&mut self) -> Option<&char> {
        self.input.peek()
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.peek() {
            if c.is_whitespace() && c != '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace();
            let c = match self.advance() {
                Some(ch) => ch,
                None => break,
            };

            let token = match c {
                '\n' | '\r' => continue,
                '+' => Token::Plus,
                '-' => Token::Minus,
                '*' => Token::Star,
                '/' => Token::Slash,
                '(' => Token::LeftParen,
                ')' => Token::RightParen,
                ',' => Token::Comma,
                '=' => Token::Equals,
                '<' => {
                    if let Some(&'=') = self.peek() {
                        self.advance();
                        Token::LessEq
                    } else if let Some(&'>') = self.peek() {
                        self.advance();
                        Token::NotEquals
                    } else {
                        Token::LessThan
                    }
                }
                '>' => {
                    if let Some(&'=') = self.peek() {
                        self.advance();
                        Token::GreaterEq
                    } else {
                        Token::GreaterThan
                    }
                }
                '"' => {
                    let mut s = String::new();
                    loop {
                        match self.advance() {
                            Some('"') => break,
                            Some(ch) => s.push(ch),
                            None => {
                                return Err(LexerError {
                                    message: "Unterminated string".to_string(),
                                    position: self.position,
                                });
                            }
                        }
                    }
                    Token::String(s)
                }
                '0'..='9' => {
                    let mut num = c.to_digit(10).unwrap() as i32;
                    while let Some(&d) = self.peek() {
                        if d.is_ascii_digit() {
                            self.advance();
                            num = num * 10 + d.to_digit(10).unwrap() as i32;
                        } else {
                            break;
                        }
                    }
                    Token::Number(num)
                }
                'A'..='Z' | 'a'..='z' => {
                    let letter = c.to_ascii_uppercase();
                    // Check if it's a keyword (only at start of token)
                    let mut keyword = String::new();
                    keyword.push(letter);
                    while let Some(&ch) = self.peek() {
                        if ch.is_ascii_alphanumeric() {
                            self.advance();
                            keyword.push(ch.to_ascii_uppercase());
                        } else {
                            break;
                        }
                    }
                    match keyword.as_str() {
                        "PRINT" => Token::Print,
                        "LET" => Token::Let,
                        "GOTO" => Token::Goto,
                        "IF" => Token::If,
                        "THEN" => Token::Then,
                        "END" => Token::End,
                        "DIM" => Token::Dim,
                        _ => {
                            // Single letter variable
                            if keyword.len() == 1 {
                                Token::Ident(keyword.chars().next().unwrap())
                            } else {
                                return Err(LexerError {
                                    message: format!("Invalid identifier: {}", keyword),
                                    position: self.position,
                                });
                            }
                        }
                    }
                }
                _ => {
                    return Err(LexerError {
                        message: format!("Unexpected character: {}", c),
                        position: self.position,
                    });
                }
            };

            tokens.push(token);
        }

        Ok(tokens)
    }
}

trait ToAsciiUpper {
    fn to_ascii_uppercase(self) -> char;
}
impl ToAsciiUpper for char {
    fn to_ascii_uppercase(self) -> char {
        if self.is_ascii_lowercase() {
            (self as u8 - 32) as char
        } else {
            self
        }
    }
}
