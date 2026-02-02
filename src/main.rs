//! Tiny BASIC Interpreter
//!
//! A minimal BASIC interpreter supporting:
//! - PRINT, LET, GOTO, IF, END, DIM
//! - Variables A-Z, integer arithmetic
//! - Commands: LOAD, SAVE, RUN, LIST, NEW, QUIT

mod ast;
mod interpreter;
mod lexer;
mod parser;

use interpreter::Interpreter;
use parser::parse;
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, BufRead, Write};

fn format_line(line: &ast::Line) -> String {
    use ast::Stmt;
    let stmt_str = match &line.stmt {
        Stmt::Print(items) => {
            let parts: Vec<String> = items
                .iter()
                .map(|p| match p {
                    ast::PrintItem::String(s) => format!("\"{}\"", s),
                    ast::PrintItem::Expr(e) => format_expr(e),
                })
                .collect();
            format!("PRINT {}", parts.join(", "))
        }
        Stmt::Let(v, e) => format!("LET {} = {}", v, format_expr(e)),
        Stmt::LetArray(v, i, e) => format!("LET {}({}) = {}", v, format_expr(i), format_expr(e)),
        Stmt::Goto(n) => format!("GOTO {}", n),
        Stmt::If { condition, then_line } => format!("IF {} THEN {}", format_expr(condition), then_line),
        Stmt::End => "END".to_string(),
        Stmt::Dim(v, s) => format!("DIM {}({})", v, s),
    };
    format!("{} {}", line.number, stmt_str)
}

fn format_expr(expr: &ast::Expr) -> String {
    use ast::{BinaryOp, Expr};
    match expr {
        Expr::Number(n) => n.to_string(),
        Expr::Variable(c) => c.to_string(),
        Expr::ArrayAccess(n, i) => format!("{}({})", n, format_expr(i)),
        Expr::Binary { left, op, right } => {
            let op_str = match op {
                BinaryOp::Add => "+",
                BinaryOp::Sub => "-",
                BinaryOp::Mul => "*",
                BinaryOp::Div => "/",
                BinaryOp::Eq => "=",
                BinaryOp::Ne => "<>",
                BinaryOp::Lt => "<",
                BinaryOp::Le => "<=",
                BinaryOp::Gt => ">",
                BinaryOp::Ge => ">=",
            };
            format!("({} {} {})", format_expr(left), op_str, format_expr(right))
        }
    }
}

fn main() {
    println!("Tiny BASIC Interpreter");
    println!("Commands: LOAD, SAVE, RUN, LIST, NEW, QUIT");
    println!();

    let mut program: BTreeMap<i32, ast::Line> = BTreeMap::new();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("> ");
        stdout.flush().unwrap();

        let mut input = String::new();
        if stdin.lock().read_line(&mut input).is_err() {
            break;
        }
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        let input_upper = input.to_uppercase();

        // Direct commands (no line number)
        if input_upper == "QUIT" || input_upper == "BYE" || input_upper == "EXIT" {
            println!("Goodbye!");
            break;
        }
        if input_upper == "NEW" {
            program.clear();
            println!("Program cleared.");
            continue;
        }
        if input_upper == "LIST" {
            if program.is_empty() {
                println!("(No program)");
            } else {
                for line in program.values() {
                    println!("{}", format_line(line));
                }
            }
            continue;
        }
        if input_upper == "RUN" {
            let lines: Vec<_> = program.values().cloned().collect();
            if lines.is_empty() {
                println!("(No program to run)");
            } else {
                match parse_program(&lines) {
                    Ok(parsed) => {
                        let mut interp = Interpreter::new(parsed);
                        if let Err(e) = interp.run() {
                            eprintln!("Runtime error: {}", e);
                        }
                    }
                    Err(e) => eprintln!("Parse error: {}", e),
                }
            }
            continue;
        }
        if input_upper.starts_with("LOAD ") {
            let path = input[5..].trim().trim_matches('"');
            match fs::read_to_string(path) {
                Ok(contents) => {
                    match parse(&contents) {
                        Ok(lines) => {
                            program.clear();
                            for line in lines {
                                program.insert(line.number, line);
                            }
                            println!("Loaded {} lines from {}", program.len(), path);
                        }
                        Err(e) => eprintln!("Parse error: {}", e),
                    }
                }
                Err(e) => eprintln!("Error loading file: {}", e),
            }
            continue;
        }
        if input_upper.starts_with("SAVE ") {
            let path = input[5..].trim().trim_matches('"');
            let mut content = String::new();
            for line in program.values() {
                content.push_str(&format_line(line));
                content.push('\n');
            }
            match fs::write(path, content) {
                Ok(_) => println!("Saved {} lines to {}", program.len(), path),
                Err(e) => eprintln!("Error saving file: {}", e),
            }
            continue;
        }

        // Try to parse as program line (NUMBER STATEMENT)
        match parse(input) {
            Ok(lines) => {
                for line in lines {
                    program.insert(line.number, line);
                }
            }
            Err(e) => {
                eprintln!("Parse error: {:?}", e);
            }
        }
    }
}

fn parse_program(lines: &[ast::Line]) -> Result<Vec<ast::Line>, parser::ParseError> {
    let mut buf = String::new();
    for line in lines {
        buf.push_str(&format_line(line));
        buf.push('\n');
    }
    parse(&buf)
}
