use crate::ast::{Error, Expr, expect_arity, expect_nonempty};
use crate::env::Env;

#[inline(always)]
pub fn builtin_eval(func: &str, e: Env, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    let list = args.into_iter().next().unwrap().into_list(func, line)?;

    if list.is_empty() {
        // Empty expression
        Ok(Expr::Sexpr(Vec::new()))
    } else {
        // Always evaluate expressions sequentially, return last result
        let mut result = Expr::Sexpr(Vec::new());
        for expr in list {
            // For each expression, if it's a List, convert to Sexpr for evaluation
            match expr {
                Expr::List(inner) => {
                    result = Expr::Sexpr(inner).eval(e, line)?;
                }
                _ => {
                    result = expr.eval(e, line)?;
                }
            }
        }
        Ok(result)
    }
}

pub fn builtin_var(e: Env, func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_nonempty(func, &args, line)?;

    // check symbols
    let symbols = args[0].clone().into_list(func, line)?;
    for s in symbols.iter() {
        if let Expr::Symbol(_) = s {
            // all gud
        } else {
            return Err(Error::IncompatibleType {
                op: func.to_string(),
                expected: "Symbol".to_string(),
                received: s.as_str(),
                line,
            });
        }
    }

    // match length
    expect_arity(func, &args, symbols.len() + 1, line)?;

    // Evaluate the values before storing them
    let mut values = Vec::new();
    for ar in args.into_iter().skip(1) {
        values.push(ar.eval(e, line)?);
    }

    for (sy, ar) in symbols.into_iter().zip(values.into_iter()) {
        let Expr::Symbol(sy) = sy else {
            return Err(Error::IncompatibleType {
                op: func.to_string(),
                expected: "Symbol".to_string(),
                received: sy.as_str(),
                line,
            });
        };
        match func {
            "def" => {
                // Insert into root environment
                let root = e.root();
                root.insert(sy, ar);
            }
            "=" => {
                // Insert into current environment
                e.insert(sy, ar);
            }
            _ => panic!(),
        }
    }

    Ok(Expr::Sexpr(Vec::new()))
}

pub fn builtin_lambda(func: &str, e: Env, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 2, line)?;

    // check arg types - accept both List and Sexpr for formals
    let formals = match args.first().unwrap() {
        Expr::List(formals) => formals.to_vec(),
        Expr::Sexpr(sexpr) => sexpr.to_vec(),
        _ => {
            return Err(Error::IncompatibleType {
                op: func.to_string(),
                expected: "List or Sexpr".to_string(),
                received: args[0].as_str(),
                line,
            });
        }
    };

    for f in formals.iter() {
        // check all formals are symbols
        let Expr::Symbol(_) = f else {
            return Err(Error::IncompatibleType {
                op: func.to_string(),
                expected: "Symbol".to_string(),
                received: f.as_str(),
                line,
            });
        };
    }

    // check body is a list or sexpr
    let body = match args.get(1).unwrap() {
        Expr::List(body) => body.to_vec(),
        Expr::Sexpr(sexpr) => sexpr.to_vec(),
        _ => {
            return Err(Error::IncompatibleType {
                op: func.to_string(),
                expected: "List or Sexpr".to_string(),
                received: args[1].as_str(),
                line,
            });
        }
    };

    // create a new environment for the lambda that captures the current environment
    let lambda_env = Env::child(e);

    Ok(Expr::Lambda {
        env: lambda_env,
        formals: formals,
        body: Box::new(Expr::List(body)), // shouldnt create new mem i think
    })
}

#[inline(always)]
pub fn builtin_if(func: &str, e: Env, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 3, line)?;
    let cond = args[0].clone().into_number(func, line)?;
    let tbr = Expr::Sexpr(args[1].clone().into_list(func, line)?);
    let fbr = Expr::Sexpr(args[2].clone().into_list(func, line)?);

    if cond != 0 {
        tbr.eval(e, line)
    } else {
        fbr.eval(e, line)
    }
}

pub fn builtin_fun(func: &str, e: Env, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 2, line)?;

    // Get the name-and-params list
    let name_and_params = match args.first().unwrap() {
        Expr::List(list) => list.to_vec(),
        Expr::Sexpr(sexpr) => sexpr.to_vec(),
        _ => {
            return Err(Error::IncompatibleType {
                op: func.to_string(),
                expected: "List or Sexpr".to_string(),
                received: args[0].as_str(),
                line,
            });
        }
    };

    if name_and_params.is_empty() {
        return Err(Error::WrongAmountOfArgs {
            func: func.to_string(),
            expected: 1,
            received: 0,
            line,
        });
    }

    // Extract function name and parameters
    let func_name = match &name_and_params[0] {
        Expr::Symbol(name) => name.clone(),
        _ => {
            return Err(Error::IncompatibleType {
                op: func.to_string(),
                expected: "Symbol".to_string(),
                received: name_and_params[0].as_str(),
                line,
            });
        }
    };

    let params = name_and_params[1..].to_vec();
    let body = args.get(1).unwrap().clone();

    // Create the lambda
    let lambda = Expr::Lambda {
        env: Env::child(e),
        formals: params,
        body: Box::new(body),
    };

    // Define the function
    let root = e.root();
    root.insert(func_name, lambda);

    Ok(Expr::Sexpr(Vec::new()))
}
