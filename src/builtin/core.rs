use crate::ast::{Error, Expr, expect_arity, expect_nonempty};
use crate::env::Env;

#[inline(always)]
pub fn builtin_eval(func: &str, e: Env, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    let qexpr = args.into_iter().next().unwrap().into_qexpr(func, line)?;

    if qexpr.is_empty() {
        // Empty expression
        Ok(Expr::Sexpr(Vec::new()))
    } else {
        // Always evaluate expressions sequentially, return last result
        let mut result = Expr::Sexpr(Vec::new());
        for expr in qexpr {
            // For each expression, if it's a Qexpr, convert to Sexpr for evaluation
            match expr {
                Expr::Qexpr(inner) => {
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
    let symbols = args[0].clone().into_qexpr(func, line)?;
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

    // check arg types
    let Expr::Qexpr(formals) = args.first().unwrap() else {
        return Err(Error::IncompatibleType {
            op: func.to_string(),
            expected: "Qexpr".to_string(),
            received: args[0].as_str(),
            line,
        });
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

    // check body is a qexpr
    let Expr::Qexpr(body) = args.get(1).unwrap() else {
        return Err(Error::IncompatibleType {
            op: func.to_string(),
            expected: "Qexpr".to_string(),
            received: args[1].as_str(),
            line,
        });
    };

    // create a new environment for the lambda that captures the current environment
    let lambda_env = Env::child(e);

    Ok(Expr::Lambda {
        env: lambda_env,
        formals: formals.to_vec(),
        body: Box::new(Expr::Qexpr(body.to_vec())), // shouldnt create new mem i think
    })
}

#[inline(always)]
pub fn builtin_if(func: &str, e: Env, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 3, line)?;
    let cond = args[0].clone().into_number(func, line)?;
    let tbr = Expr::Sexpr(args[1].clone().into_qexpr(func, line)?);
    let fbr = Expr::Sexpr(args[2].clone().into_qexpr(func, line)?);

    if cond != 0 {
        tbr.eval(e, line)
    } else {
        fbr.eval(e, line)
    }
}
