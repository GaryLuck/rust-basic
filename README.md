# Tiny BASIC Interpreter

A minimal BASIC interpreter implemented in Rust, supporting integer arithmetic, variables, arrays, and control flow.

## Requirements

- Rust (edition 2021)

## Building and Running

```bash
cargo build
cargo run
```

## Commands

| Command | Description |
|---------|-------------|
| `LOAD "file.bas"` | Load a program from a file |
| `SAVE "file.bas"` | Save the current program to a file |
| `RUN` | Execute the loaded program |
| `LIST` | Display the current program |
| `NEW` | Clear the program |
| `QUIT` | Exit the interpreter |

## Language Features

### Statements

| Statement | Example | Description |
|-----------|---------|-------------|
| `PRINT` | `10 PRINT "Hello", X, A(I)` | Print comma-separated values and string literals |
| `LET` | `20 LET X = 5` | Assign a value to a variable |
| `LET` (array) | `30 LET A(I) = 10` | Assign a value to an array element |
| `GOTO` | `40 GOTO 100` | Jump to a line number |
| `IF` | `50 IF X < 10 THEN 70` | Conditional jump |
| `END` | `60 END` | End of program |
| `DIM` | `5 DIM A(10)` | Declare an array (indices 0 to size-1) |

### Variables

- Single letters **A-Z**
- Initialized to 0
- Integer arithmetic only

### Expressions

- Arithmetic: `+`, `-`, `*`, `/`
- Comparisons: `=`, `<>`, `<`, `<=`, `>`, `>=`
- Variables: `X`, `A(I)`
- Parentheses for grouping

## Example Program

```
10 PRINT "Hello, Tiny BASIC!"
20 LET X = 5
30 LET Y = 10
40 PRINT "X + Y =", X + Y
50 IF X < Y THEN 70
60 GOTO 80
70 PRINT "X is less than Y"
80 END
```

## Sample Files

- `sample.bas` - Basic arithmetic and control flow
- `arrays.bas` - Array declaration and indexing
