use crate::ast::{Error, Expr, expect_arity};

/// Helper function to truncate float to int if it's close enough to an integer
fn truncate_float(f: f32) -> Expr {
    Expr::Number(f.floor() as i32)
}

pub fn builtin_truncate(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    match args[0] {
        Expr::Number(n) => Ok(Expr::Number(n)),
        Expr::Float(f) => Ok(truncate_float(f)),
        _ => Err(Error::IncompatibleType {
            op: func.to_string(),
            expected: "Number or Float".to_string(),
            received: args[0].as_str(),
            line,
        }),
    }
}

pub fn builtin_sqrt(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    match args[0] {
        Expr::Number(n) => {
            if n < 0 {
                return Err(Error::ParseError {
                    msg: "sqrt argument must be non-negative".to_string(),
                    line,
                });
            }
            let result = (n as f32).sqrt();
            Ok(Expr::Float(result))
        }
        Expr::Float(f) => {
            if f < 0.0 {
                return Err(Error::ParseError {
                    msg: "sqrt argument must be non-negative".to_string(),
                    line,
                });
            }
            let result = f.sqrt();
            Ok(Expr::Float(result))
        }
        _ => Err(Error::IncompatibleType {
            op: func.to_string(),
            expected: "Number or Float".to_string(),
            received: args[0].as_str(),
            line,
        }),
    }
}

pub fn builtin_abs(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    match args[0] {
        Expr::Number(n) => Ok(Expr::Number(n.abs())),
        Expr::Float(f) => Ok(Expr::Float(f.abs())),
        _ => Err(Error::IncompatibleType {
            op: func.to_string(),
            expected: "Number or Float".to_string(),
            received: args[0].as_str(),
            line,
        }),
    }
}
pub fn builtin_min(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 2, line)?;
    match (&args[0], &args[1]) {
        (Expr::Number(l), Expr::Number(r)) => Ok(Expr::Number((*l).min(*r))),
        (Expr::Float(l), Expr::Float(r)) => Ok(Expr::Float((*l).min(*r))),
        (Expr::Number(l), Expr::Float(r)) => Ok(Expr::Float((*l as f32).min(*r))),
        (Expr::Float(l), Expr::Number(r)) => Ok(Expr::Float(l.min(*r as f32))),
        _ => Err(Error::IncompatibleType {
            op: func.to_string(),
            expected: "Number or Float".to_string(),
            received: args[0].as_str(),
            line,
        }),
    }
}

pub fn builtin_max(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 2, line)?;
    match (&args[0], &args[1]) {
        (Expr::Number(l), Expr::Number(r)) => Ok(Expr::Number((*l).max(*r))),
        (Expr::Float(l), Expr::Float(r)) => Ok(Expr::Float((*l).max(*r))),
        (Expr::Number(l), Expr::Float(r)) => Ok(Expr::Float((*l as f32).max(*r))),
        (Expr::Float(l), Expr::Number(r)) => Ok(Expr::Float(l.max(*r as f32))),
        _ => Err(Error::IncompatibleType {
            op: func.to_string(),
            expected: "Number or Float".to_string(),
            received: args[0].as_str(),
            line,
        }),
    }
}

pub fn builtin_floor(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    match args[0] {
        Expr::Number(n) => Ok(Expr::Float(n as f32)),
        Expr::Float(f) => Ok(Expr::Float(f.floor())),
        _ => Err(Error::IncompatibleType {
            op: func.to_string(),
            expected: "Number or Float".to_string(),
            received: args[0].as_str(),
            line,
        }),
    }
}

pub fn builtin_ceil(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    match args[0] {
        Expr::Number(n) => Ok(Expr::Float(n as f32)),
        Expr::Float(f) => Ok(Expr::Float(f.ceil())),
        _ => Err(Error::IncompatibleType {
            op: func.to_string(),
            expected: "Number or Float".to_string(),
            received: args[0].as_str(),
            line,
        }),
    }
}

pub fn builtin_round(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    match args[0] {
        Expr::Number(n) => Ok(Expr::Float(n as f32)),
        Expr::Float(f) => Ok(Expr::Float(f.round())),
        _ => Err(Error::IncompatibleType {
            op: func.to_string(),
            expected: "Number or Float".to_string(),
            received: args[0].as_str(),
            line,
        }),
    }
}

pub fn builtin_sin(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    match args[0] {
        Expr::Number(n) => {
            let result = (n as f32).sin();
            Ok(Expr::Float(result))
        }
        Expr::Float(f) => {
            let result = f.sin();
            Ok(Expr::Float(result))
        }
        _ => Err(Error::IncompatibleType {
            op: func.to_string(),
            expected: "Number or Float".to_string(),
            received: args[0].as_str(),
            line,
        }),
    }
}

pub fn builtin_cos(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    match args[0] {
        Expr::Number(n) => {
            let result = (n as f32).cos();
            Ok(Expr::Float(result))
        }
        Expr::Float(f) => {
            let result = f.cos();
            Ok(Expr::Float(result))
        }
        _ => Err(Error::IncompatibleType {
            op: func.to_string(),
            expected: "Number or Float".to_string(),
            received: args[0].as_str(),
            line,
        }),
    }
}

pub fn builtin_tan(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    match args[0] {
        Expr::Number(n) => {
            let result = (n as f32).tan();
            Ok(Expr::Float(result))
        }
        Expr::Float(f) => {
            let result = f.tan();
            Ok(Expr::Float(result))
        }
        _ => Err(Error::IncompatibleType {
            op: func.to_string(),
            expected: "Number or Float".to_string(),
            received: args[0].as_str(),
            line,
        }),
    }
}

pub fn builtin_log(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    match args[0] {
        Expr::Number(n) => {
            if n <= 0 {
                return Err(Error::ParseError {
                    msg: "log argument must be positive".to_string(),
                    line,
                });
            }
            let result = (n as f32).ln();
            Ok(Expr::Float(result))
        }
        Expr::Float(f) => {
            if f <= 0.0 {
                return Err(Error::ParseError {
                    msg: "log argument must be positive".to_string(),
                    line,
                });
            }
            let result = f.ln();
            Ok(Expr::Float(result))
        }
        _ => Err(Error::IncompatibleType {
            op: func.to_string(),
            expected: "Number or Float".to_string(),
            received: args[0].as_str(),
            line,
        }),
    }
}

pub fn builtin_exp(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    match args[0] {
        Expr::Number(n) => {
            let result = (n as f32).exp();
            Ok(Expr::Float(result))
        }
        Expr::Float(f) => {
            let result = f.exp();
            Ok(Expr::Float(result))
        }
        _ => Err(Error::IncompatibleType {
            op: func.to_string(),
            expected: "Number or Float".to_string(),
            received: args[0].as_str(),
            line,
        }),
    }
}
