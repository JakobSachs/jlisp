use crate::ast::{Error, Expr, expect_arity};

pub fn builtin_range(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    let a = args.into_iter().next().unwrap();
    let Expr::Number(rng) = a else {
        return Err(Error::IncompatibleType {
            op: func.to_string(),
            expected: "Number".to_string(),
            received: a.as_str(),
            line,
        });
    };

    Ok(Expr::List((0..rng).map(Expr::Number).collect()))
}

pub fn builtin_sort(func: &str, mut args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;

    let mut nums: Vec<i32> = args
        .remove(0)
        .into_list(func, line)?
        .iter()
        .map(|e| match e {
            Expr::Number(i) => Ok(*i),
            _ => Err(Error::IncompatibleType {
                op: func.to_string(),
                expected: "Number".to_string(),
                received: e.as_str(),
                line,
            }),
        })
        .collect::<Result<Vec<_>, Error>>()?; // returns on error
    nums.sort();
    Ok(Expr::List(nums.iter().map(|i| Expr::Number(*i)).collect()))
}

pub fn builtin_print(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    let a = args.into_iter().next().unwrap();
    println!("{}", a);
    Ok(Expr::Sexpr(Vec::new()))
}
