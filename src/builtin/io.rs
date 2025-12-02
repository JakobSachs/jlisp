use crate::ast::{Error, Expr, expect_arity};
use crate::builtin::macros::single_string_op;
use crate::env::Env;
use std::fs;

pub fn builtin_load(func: &str, e: Env, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    single_string_op!(
        args,
        func,
        line,
        |path: String, _func: &str, line: usize| {
            let contents = fs::read_to_string(&path).map_err(|err| Error::IoError {
                msg: format!("Failed to load file '{}': {}", path, err),
                line,
            })?;

            let parser = crate::grammar::JLispParser::new();
            let result = parser.parse(&contents).map_err(|err| Error::ParseError {
                msg: format!("Failed to parse file '{}': {}", path, err),
                line,
            })?;

            let mut last = Expr::Sexpr(vec![]);
            for expr in result.exprs {
                last = expr.eval(e, line)?;
            }

            Ok(last)
        }
    )
}

pub fn builtin_read(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    single_string_op!(
        args,
        func,
        line,
        |path: String, _func: &str, line: usize| {
            let contents = fs::read_to_string(&path).map_err(|err| Error::IoError {
                msg: format!("Failed to load file '{}': {}", path, err),
                line,
            })?;
            Ok(Expr::String(contents))
        }
    )
}

pub fn builtin_error(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    let msg = args[0].clone().into_string(func, line)?;
    Err(Error::ParseError { msg, line })
}
