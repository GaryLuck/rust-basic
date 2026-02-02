/// Interpreter for Tiny BASIC - executes parsed programs
use crate::ast::{BinaryOp, Expr, Line, PrintItem, Stmt};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
#[allow(dead_code)]
pub enum RuntimeError {
    DivisionByZero,
    UndefinedVariable(char),
    UndefinedArray(char),
    ArrayNotDimensioned(char),
    InvalidLineNumber(i32),
    IndexOutOfBounds { array: char, index: i32, size: i32 },
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::DivisionByZero => write!(f, "Division by zero"),
            RuntimeError::UndefinedVariable(c) => write!(f, "Undefined variable: {}", c),
            RuntimeError::UndefinedArray(c) => write!(f, "Undefined array: {}", c),
            RuntimeError::ArrayNotDimensioned(c) => write!(f, "Array {} not dimensioned", c),
            RuntimeError::InvalidLineNumber(n) => write!(f, "Invalid line number: {}", n),
            RuntimeError::IndexOutOfBounds { array, index, size } => {
                write!(f, "Index {} out of bounds for array {} (size {})", index, array, size)
            }
        }
    }
}

pub struct Interpreter {
    variables: HashMap<char, i32>,
    arrays: HashMap<char, Vec<i32>>,
    program: Vec<Line>,
    line_index: usize,
    done: bool,
}

impl Interpreter {
    pub fn new(program: Vec<Line>) -> Self {
        let mut interp = Self {
            variables: HashMap::new(),
            arrays: HashMap::new(),
            program,
            line_index: 0,
            done: false,
        };
        // Initialize all variables A-Z to 0
        for c in 'A'..='Z' {
            interp.variables.insert(c, 0);
        }
        interp
    }

    fn get_line_index(&self, line_num: i32) -> Result<usize, RuntimeError> {
        self.program
            .iter()
            .position(|l| l.number == line_num)
            .ok_or(RuntimeError::InvalidLineNumber(line_num))
    }

    fn eval_expr(&self, expr: &Expr) -> Result<i32, RuntimeError> {
        match expr {
            Expr::Number(n) => Ok(*n),
            Expr::Variable(c) => self
                .variables
                .get(c)
                .copied()
                .ok_or(RuntimeError::UndefinedVariable(*c)),
            Expr::ArrayAccess(name, index_expr) => {
                let index = self.eval_expr(index_expr)?;
                let arr = self
                    .arrays
                    .get(name)
                    .ok_or(RuntimeError::ArrayNotDimensioned(*name))?;
                if index < 0 || index >= arr.len() as i32 {
                    return Err(RuntimeError::IndexOutOfBounds {
                        array: *name,
                        index,
                        size: arr.len() as i32,
                    });
                }
                Ok(arr[index as usize])
            }
            Expr::Binary { left, op, right } => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                match op {
                    BinaryOp::Add => Ok(l + r),
                    BinaryOp::Sub => Ok(l - r),
                    BinaryOp::Mul => Ok(l * r),
                    BinaryOp::Div => {
                        if r == 0 {
                            Err(RuntimeError::DivisionByZero)
                        } else {
                            Ok(l / r)
                        }
                    }
                    BinaryOp::Eq => Ok((l == r) as i32),
                    BinaryOp::Ne => Ok((l != r) as i32),
                    BinaryOp::Lt => Ok((l < r) as i32),
                    BinaryOp::Le => Ok((l <= r) as i32),
                    BinaryOp::Gt => Ok((l > r) as i32),
                    BinaryOp::Ge => Ok((l >= r) as i32),
                }
            }
        }
    }

    fn execute_statement(&mut self, stmt: &Stmt) -> Result<Option<i32>, RuntimeError> {
        match stmt {
            Stmt::Print(items) => {
                let mut output = Vec::new();
                for item in items {
                    match item {
                        PrintItem::String(s) => output.push(s.clone()),
                        PrintItem::Expr(expr) => output.push(self.eval_expr(expr)?.to_string()),
                    }
                }
                println!("{}", output.join(" "));
                Ok(None)
            }
            Stmt::Let(var, value) => {
                let val = self.eval_expr(value)?;
                self.variables.insert(*var, val);
                Ok(None)
            }
            Stmt::LetArray(name, index_expr, value) => {
                let index = self.eval_expr(index_expr)?;
                let val = self.eval_expr(value)?;
                let arr = self
                    .arrays
                    .get_mut(name)
                    .ok_or(RuntimeError::ArrayNotDimensioned(*name))?;
                if index < 0 || index >= arr.len() as i32 {
                    return Err(RuntimeError::IndexOutOfBounds {
                        array: *name,
                        index,
                        size: arr.len() as i32,
                    });
                }
                arr[index as usize] = val;
                Ok(None)
            }
            Stmt::Goto(line_num) => Ok(Some(*line_num)),
            Stmt::If { condition, then_line } => {
                let result = self.eval_expr(condition)?;
                if result != 0 {
                    Ok(Some(*then_line))
                } else {
                    Ok(None)
                }
            }
            Stmt::End => {
                self.done = true;
                Ok(None)
            }
            Stmt::Dim(name, size) => {
                if *size < 0 {
                    return Err(RuntimeError::IndexOutOfBounds {
                        array: *name,
                        index: *size,
                        size: 0,
                    });
                }
                self.arrays
                    .insert(*name, vec![0; *size as usize]);
                Ok(None)
            }
        }
    }

    pub fn run(&mut self) -> Result<(), RuntimeError> {
        if self.program.is_empty() {
            return Ok(());
        }

        self.line_index = 0;
        self.done = false;

        while !self.done && self.line_index < self.program.len() {
            let stmt = self.program[self.line_index].stmt.clone();
            if let Some(goto_line) = self.execute_statement(&stmt)? {
                self.line_index = self.get_line_index(goto_line)?;
            } else {
                self.line_index += 1;
            }
        }

        Ok(())
    }
}
