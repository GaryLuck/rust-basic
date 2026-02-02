/// Abstract Syntax Tree for Tiny BASIC programs

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i32),
    Variable(char),
    ArrayAccess(char, Box<Expr>),
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Print(Vec<PrintItem>),
    Let(char, Box<Expr>),
    LetArray(char, Box<Expr>, Box<Expr>),
    Goto(i32),
    If {
        condition: Box<Expr>,
        then_line: i32,
    },
    End,
    Dim(char, i32),
}

#[derive(Debug, Clone)]
pub enum PrintItem {
    Expr(Expr),
    String(String),
}

#[derive(Debug, Clone)]
pub struct Line {
    pub number: i32,
    pub stmt: Stmt,
}
