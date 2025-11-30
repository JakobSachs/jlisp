use crate::ast::{Error, Expr, expect_arity};

pub fn builtin_chars(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    let s = args[0].clone().into_string(func, line)?;
    Ok(Expr::Qexpr(s.chars().map(Expr::Char).collect()))
}

pub fn builtin_int(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    match args[0].clone().into_string(func, line)?.parse::<i32>() {
        Ok(v) => Ok(Expr::Number(v)),
        Err(_) => Err(Error::ParseError {
            msg: "couldn't parse int from string".to_string(),
            line,
        }),
    }
}

pub fn builtin_str_sub(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 3, line)?;
    let s = args[0].clone().into_string(func, line)?;
    let start = args[1].clone().into_number(func, line)?;
    let end = args[2].clone().into_number(func, line)?;

    if start < 0 || end < 0 {
        return Err(Error::ParseError {
            msg: "substring indices must be non-negative".to_string(),
            line,
        });
    }

    if start > end {
        return Err(Error::ParseError {
            msg: "substring start index must be <= end index".to_string(),
            line,
        });
    }

    let chars: Vec<char> = s.chars().collect();
    let start_usize = start as usize;
    let end_usize = end as usize;

    if end_usize > chars.len() {
        return Err(Error::ParseError {
            msg: "substring end index out of bounds".to_string(),
            line,
        });
    }

    let substring: String = chars[start_usize..end_usize].iter().collect();
    Ok(Expr::String(substring))
}
