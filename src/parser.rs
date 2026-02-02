/// Parser for Tiny BASIC - builds AST from tokens
use crate::ast::{BinaryOp, Expr, Line, PrintItem, Stmt};
use crate::lexer::{Lexer, LexerError, Token};
use std::fmt;
use std::iter::Peekable;
use std::vec::IntoIter;

#[derive(Debug)]
#[allow(dead_code)]
pub enum ParseError {
    Lexer(LexerError),
    UnexpectedEnd,
    UnexpectedToken(String),
    InvalidLineNumber,
}

impl From<LexerError> for ParseError {
    fn from(e: LexerError) -> Self {
        ParseError::Lexer(e)
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Lexer(e) => write!(f, "{}", e),
            ParseError::UnexpectedEnd => write!(f, "Unexpected end of input"),
            ParseError::UnexpectedToken(s) => write!(f, "{}", s),
            ParseError::InvalidLineNumber => write!(f, "Invalid line number"),
        }
    }
}

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens.into_iter().peekable(),
        }
    }

    fn advance(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    fn parse_line(&mut self) -> Result<Option<Line>, ParseError> {
        // Line format: NUMBER STATEMENT
        let line_num = match self.peek().cloned() {
            Some(Token::Number(n)) => {
                self.advance();
                n
            }
            Some(_) => return Ok(None),
            None => return Ok(None),
        };

        let stmt = self.parse_statement()?;
        Ok(Some(Line {
            number: line_num,
            stmt,
        }))
    }

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        match self.advance() {
            Some(Token::Print) => self.parse_print(),
            Some(Token::Let) => self.parse_let(),
            Some(Token::Goto) => self.parse_goto(),
            Some(Token::If) => self.parse_if(),
            Some(Token::End) => Ok(Stmt::End),
            Some(Token::Dim) => self.parse_dim(),
            Some(t) => Err(ParseError::UnexpectedToken(format!("Expected statement, got {:?}", t))),
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    fn parse_print(&mut self) -> Result<Stmt, ParseError> {
        let mut items = Vec::new();

        loop {
            match self.peek().cloned() {
                Some(Token::String(s)) => {
                    self.advance();
                    items.push(PrintItem::String(s));
                }
                Some(Token::Ident(_)) | Some(Token::Number(_)) | Some(Token::LeftParen) => {
                    items.push(PrintItem::Expr(self.parse_expr()?));
                }
                Some(Token::Comma) => {
                    self.advance();
                    continue;
                }
                _ => break,
            }

            if matches!(self.peek(), Some(Token::Comma)) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(Stmt::Print(items))
    }

    fn parse_let(&mut self) -> Result<Stmt, ParseError> {
        let var = match self.advance() {
            Some(Token::Ident(c)) => c,
            Some(t) => return Err(ParseError::UnexpectedToken(format!("Expected variable, got {:?}", t))),
            None => return Err(ParseError::UnexpectedEnd),
        };

        if matches!(self.peek(), Some(Token::LeftParen)) {
            // Array assignment: LET A(I) = expr
            self.advance();
            let index = self.parse_expr()?;
            self.expect_token(Token::RightParen)?;
            self.expect_token(Token::Equals)?;
            let value = self.parse_expr()?;
            Ok(Stmt::LetArray(var, Box::new(index), Box::new(value)))
        } else {
            self.expect_token(Token::Equals)?;
            let value = self.parse_expr()?;
            Ok(Stmt::Let(var, Box::new(value)))
        }
    }

    fn expect_token(&mut self, expected: Token) -> Result<(), ParseError> {
        match self.advance() {
            Some(t) if std::mem::discriminant(&t) == std::mem::discriminant(&expected) => Ok(()),
            Some(t) => Err(ParseError::UnexpectedToken(format!("Expected {:?}, got {:?}", expected, t))),
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    fn parse_goto(&mut self) -> Result<Stmt, ParseError> {
        let line = match self.advance() {
            Some(Token::Number(n)) => n,
            Some(t) => return Err(ParseError::UnexpectedToken(format!("Expected line number, got {:?}", t))),
            None => return Err(ParseError::UnexpectedEnd),
        };
        Ok(Stmt::Goto(line))
    }

    fn parse_if(&mut self) -> Result<Stmt, ParseError> {
        let condition = self.parse_expr()?;
        self.expect_token(Token::Then)?;
        let then_line = match self.advance() {
            Some(Token::Number(n)) => n,
            Some(t) => return Err(ParseError::UnexpectedToken(format!("Expected line number, got {:?}", t))),
            None => return Err(ParseError::UnexpectedEnd),
        };
        Ok(Stmt::If {
            condition: Box::new(condition),
            then_line,
        })
    }

    fn parse_dim(&mut self) -> Result<Stmt, ParseError> {
        let var = match self.advance() {
            Some(Token::Ident(c)) => c,
            Some(t) => return Err(ParseError::UnexpectedToken(format!("Expected array name, got {:?}", t))),
            None => return Err(ParseError::UnexpectedEnd),
        };
        self.expect_token(Token::LeftParen)?;
        let size = match self.advance() {
            Some(Token::Number(n)) => n,
            Some(t) => return Err(ParseError::UnexpectedToken(format!("Expected array size, got {:?}", t))),
            None => return Err(ParseError::UnexpectedEnd),
        };
        self.expect_token(Token::RightParen)?;
        Ok(Stmt::Dim(var, size))
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_additive()?;
        if let Some(op_token) = self.peek() {
            let op = match op_token {
                Token::Equals => BinaryOp::Eq,
                Token::NotEquals => BinaryOp::Ne,
                Token::LessThan => BinaryOp::Lt,
                Token::LessEq => BinaryOp::Le,
                Token::GreaterThan => BinaryOp::Gt,
                Token::GreaterEq => BinaryOp::Ge,
                _ => return Ok(left),
            };
            self.advance();
            let right = self.parse_additive()?;
            return Ok(Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            });
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplicative()?;
        while let Some(op_token) = self.peek() {
            let op = match op_token {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;
        while let Some(op_token) = self.peek() {
            let op = match op_token {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if matches!(self.peek(), Some(Token::Minus)) {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expr::Binary {
                left: Box::new(Expr::Number(0)),
                op: BinaryOp::Sub,
                right: Box::new(expr),
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.advance() {
            Some(Token::Number(n)) => Ok(Expr::Number(n)),
            Some(Token::Ident(c)) => {
                if matches!(self.peek(), Some(Token::LeftParen)) {
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect_token(Token::RightParen)?;
                    Ok(Expr::ArrayAccess(c, Box::new(index)))
                } else {
                    Ok(Expr::Variable(c))
                }
            }
            Some(Token::LeftParen) => {
                let expr = self.parse_expr()?;
                self.expect_token(Token::RightParen)?;
                Ok(expr)
            }
            Some(t) => Err(ParseError::UnexpectedToken(format!("Expected expression, got {:?}", t))),
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    pub fn parse_program(&mut self) -> Result<Vec<Line>, ParseError> {
        let mut lines = Vec::new();
        loop {
            match self.parse_line()? {
                Some(line) => lines.push(line),
                None => break,
            }
        }
        lines.sort_by_key(|l| l.number);
        Ok(lines)
    }
}

pub fn parse(source: &str) -> Result<Vec<Line>, ParseError> {
    let tokens = Lexer::new(source).tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}
