use crate::ast::{Error, Expr, expect_arity};

/// Helper function to create incompatible type errors
fn incompatible_type_error(func: &str, expected: &str, received: String, line: usize) -> Result<Expr, Error> {
    Err(Error::IncompatibleType {
        op: func.to_string(),
        expected: expected.to_string(),
        received,
        line,
    })
}

/// Macro for unary math operations that return Float
macro_rules! unary_math_op {
    ($func_name:ident, $rust_op:ident) => {
        pub fn $func_name(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
            expect_arity(func, &args, 1, line)?;
            match args[0] {
                Expr::Number(n) => Ok(Expr::Float((n as f32).$rust_op())),
                Expr::Float(f) => Ok(Expr::Float(f.$rust_op())),
                _ => incompatible_type_error(func, "Number or Float", args[0].as_str(), line),
            }
        }
    };
}

/// Macro for unary math operations with validation
macro_rules! unary_math_op_with_validation {
    ($func_name:ident, $rust_op:ident, $validation:expr, $error_msg:expr) => {
        pub fn $func_name(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
            expect_arity(func, &args, 1, line)?;
            match args[0] {
                Expr::Number(n) => {
                    if !$validation(n as f32) {
                        return Err(Error::ParseError {
                            msg: $error_msg.to_string(),
                            line,
                        });
                    }
                    let result = (n as f32).$rust_op();
                    Ok(Expr::Float(result))
                }
                Expr::Float(f) => {
                    if !$validation(f) {
                        return Err(Error::ParseError {
                            msg: $error_msg.to_string(),
                            line,
                        });
                    }
                    let result = f.$rust_op();
                    Ok(Expr::Float(result))
                }
                _ => incompatible_type_error(func, "Number or Float", args[0].as_str(), line),
            }
        }
    };
}

/// Macro for binary math operations
macro_rules! binary_math_op {
    ($func_name:ident, $rust_op:ident) => {
        pub fn $func_name(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
            expect_arity(func, &args, 2, line)?;
            match (&args[0], &args[1]) {
                (Expr::Number(l), Expr::Number(r)) => Ok(Expr::Number((*l).$rust_op(*r))),
                (Expr::Float(l), Expr::Float(r)) => Ok(Expr::Float((*l).$rust_op(*r))),
                (Expr::Number(l), Expr::Float(r)) => Ok(Expr::Float((*l as f32).$rust_op(*r))),
                (Expr::Float(l), Expr::Number(r)) => Ok(Expr::Float(l.$rust_op(*r as f32))),
                _ => incompatible_type_error(func, "Number or Float", args[0].as_str(), line),
            }
        }
    };
}

/// Helper function to truncate float to int if it's close enough to an integer
fn truncate_float(f: f32) -> Expr {
    Expr::Number(f.floor() as i32)
}

pub fn builtin_truncate(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    match args[0] {
        Expr::Number(n) => Ok(Expr::Number(n)),
        Expr::Float(f) => Ok(truncate_float(f)),
        _ => incompatible_type_error(func, "Number or Float", args[0].as_str(), line),
    }
}

// Generate sqrt with validation for non-negative arguments
unary_math_op_with_validation!(builtin_sqrt, sqrt, |x| x >= 0.0, "sqrt argument must be non-negative");

pub fn builtin_abs(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    match args[0] {
        Expr::Number(n) => Ok(Expr::Number(n.abs())),
        Expr::Float(f) => Ok(Expr::Float(f.abs())),
        _ => incompatible_type_error(func, "Number or Float", args[0].as_str(), line),
    }
}
// Generate binary operations
binary_math_op!(builtin_min, min);
binary_math_op!(builtin_max, max);

// Generate rounding operations
unary_math_op!(builtin_floor, floor);
unary_math_op!(builtin_ceil, ceil);
unary_math_op!(builtin_round, round);

// Generate trigonometric operations
unary_math_op!(builtin_sin, sin);
unary_math_op!(builtin_cos, cos);
unary_math_op!(builtin_tan, tan);

// Generate logarithmic and exponential operations
unary_math_op_with_validation!(builtin_log, ln, |x| x > 0.0, "log argument must be positive");
unary_math_op!(builtin_exp, exp);
