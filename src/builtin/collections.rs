use crate::ast::{Error, Expr, expect_arity, expect_nonempty};
use crate::builtin::macros::single_list_op;

pub fn builtin_head(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    single_list_op!(args, func, line, |ls: Vec<Expr>| Ok(Expr::List(vec![
        ls.into_iter().next().unwrap()
    ])))
}

pub fn builtin_last(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    single_list_op!(args, func, line, |ls: Vec<Expr>| Ok(ls
        .into_iter()
        .last()
        .unwrap()))
}

pub fn builtin_tail(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    single_list_op!(args, func, line, |ls: Vec<Expr>| Ok(Expr::List(
        ls.into_iter().skip(1).collect()
    )))
}

pub fn builtin_list(_func: &str, args: Vec<Expr>, _line: usize) -> Result<Expr, Error> {
    Ok(Expr::List(args))
}

pub fn builtin_join(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    if args.is_empty() {
        return Ok(Expr::List(Vec::new()));
    }

    // Check if all arguments are strings
    let all_strings = args.iter().all(|arg| matches!(arg, Expr::String(_)));
    let all_lists = args.iter().all(|arg| matches!(arg, Expr::List(_)));

    if all_strings {
        // String concatenation mode
        let mut result = String::new();
        for arg in args {
            let s = arg.into_string(func, line)?;
            result.push_str(&s);
        }
        Ok(Expr::String(result))
    } else if all_lists {
        // List concatenation mode (original behavior)
        let mut out = Vec::new();
        for q in args {
            out.extend(q.into_list(func, line)?);
        }
        Ok(Expr::List(out))
    } else {
        // Mixed types - error
        Err(Error::InconsistentTypes {
            op: func.to_string(),
            line,
        })
    }
}

pub fn builtin_len(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    let arg = args.into_iter().next().unwrap();

    match arg {
        Expr::String(s) => Ok(Expr::Number(s.chars().count() as i32)),
        Expr::List(q) => Ok(Expr::Number(q.len() as i32)),
        Expr::Sexpr(s) => Ok(Expr::Number(s.len() as i32)),
        _ => Err(Error::IncompatibleType {
            op: func.to_string(),
            expected: "String, List, or Sexpr".to_string(),
            received: arg.as_str(),
            line,
        }),
    }
}

pub fn builtin_split(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 2, line)?;
    let delimiter = args[0].clone();
    let input = args[1].clone();

    match input {
        Expr::String(s) => {
            let Expr::Char(delimiter_char) = delimiter else {
                return Err(Error::IncompatibleType {
                    op: func.to_string(),
                    expected: "Char".to_string(),
                    received: delimiter.as_str(),
                    line,
                });
            };

            let parts: Vec<Expr> = s
                .split(delimiter_char)
                .map(|part| Expr::String(part.to_string()))
                .collect();
            Ok(Expr::List(parts))
        }
        Expr::List(q) => {
            let mut result = Vec::new();
            let mut current_chunk = Vec::new();

            for item in q {
                if item == delimiter {
                    result.push(Expr::List(current_chunk.clone()));
                    current_chunk.clear();
                } else {
                    current_chunk.push(item);
                }
            }
            result.push(Expr::List(current_chunk));

            Ok(Expr::List(result))
        }
        _ => Err(Error::IncompatibleType {
            op: func.to_string(),
            expected: "String or List".to_string(),
            received: input.as_str(),
            line,
        }),
    }
}
