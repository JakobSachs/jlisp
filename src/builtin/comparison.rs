use crate::ast::{Error, Expr, expect_arity};
use crate::builtin::macros::two_number_op;

pub fn builtin_comp(func: &str, mut args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 2, line)?;
    let a = args.remove(0);
    let b = args.remove(0);

    let o = match func {
        "==" => {
            if a == b {
                1
            } else {
                0
            }
        }
        "!=" => {
            if a == b {
                0
            } else {
                1
            }
        }
        _ => panic!(),
    };
    Ok(Expr::Number(o))
}

pub fn builtin_logic(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    match func {
        "not" => {
            expect_arity(func, &args, 1, line)?;
            let arg = args[0].clone().into_number(func, line)?;
            Ok(Expr::Number(if arg == 0 { 1 } else { 0 }))
        }
        "and" => {
            if args.is_empty() {
                return Ok(Expr::Number(1));
            }
            for arg in &args {
                let val = arg.clone().into_number(func, line)?;
                if val == 0 {
                    return Ok(Expr::Number(0));
                }
            }
            Ok(Expr::Number(1))
        }
        "or" => {
            if args.is_empty() {
                return Ok(Expr::Number(0));
            }
            for arg in &args {
                let val = arg.clone().into_number(func, line)?;
                if val != 0 {
                    return Ok(Expr::Number(1));
                }
            }
            Ok(Expr::Number(0))
        }
        _ => panic!(),
    }
}

pub fn builtin_ord(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    two_number_op!(args, func, line, |l, r| {
        match func {
            ">" => l > r,
            "<" => l < r,
            ">=" => l >= r,
            "<=" => l <= r,
            _ => panic!(),
        }
    })
}
