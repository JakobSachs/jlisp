use crate::ast::{Error, Expr, expect_arity, expect_nonempty};
use crate::env::Env;
use std::fs;
use std::mem;

macro_rules! single_qexpr_op {
    ($args:expr, $func:expr, $line:expr, $op:expr) => {
        {
            expect_arity($func, &$args, 1, $line)?;
            let ls = $args.into_iter().next().unwrap().into_qexpr($func, $line)?;
            expect_nonempty($func, &ls, $line)?;
            $op(ls)
        }
    };
}

macro_rules! two_number_op {
    ($args:expr, $func:expr, $line:expr, $op:expr) => {
        {
            expect_arity($func, &$args, 2, $line)?;
            let l = $args[0].clone().into_number($func, $line)?;
            let r = $args[1].clone().into_number($func, $line)?;
            Ok(Expr::Number($op(l, r) as i32))
        }
    };
}

macro_rules! single_string_op {
    ($args:expr, $func:expr, $line:expr, $op:expr) => {
        {
            expect_arity($func, &$args, 1, $line)?;
            let s = $args.into_iter().next().unwrap().into_string($func, $line)?;
            $op(s, $func, $line)
        }
    };
}

fn _builtin_op(sym: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    //check type of first member is valid
    if !matches!(args[0], Expr::Number(_) | Expr::Float(_) | Expr::Char(_)) {
        return Err(Error::IncompatibleType {
            op: sym.to_owned(),
            expected: "Number,Float,Char".to_string(),
            received: args[0].as_str(),
            line,
        });
    }

    // check all for same type (not content)
    if args.len() > 1
        && !args.first().is_none_or(|first| {
            let first_type = mem::discriminant(first);
            args.iter().all(|x| mem::discriminant(x) == first_type)
        })
    {
        return Err(Error::InconsistentTypes {
            op: sym.to_owned(),
            line,
        });
    }

    if sym == "-" && args.len() == 1 {
        match args[0] {
            Expr::Number(n) => return Ok(Expr::Number(-n)),
            Expr::Float(f) => return Ok(Expr::Float(-f)),
            _ => {
                return Err(Error::IncompatibleType {
                    op: sym.to_owned(),
                    expected: "Number,Float".to_string(),
                    received: args[0].as_str(),
                    line,
                });
            }
        }
    }

    match args[0] {
        Expr::Number(start) => {
            let func: fn(i32, i32) -> i32 = match sym {
                "+" => |a, b| a + b,
                "-" => |a, b| a - b,
                "*" => |a, b| a * b,
                "/" => |a, b| a / b,
                _ => panic!(),
            };
            let mut out = start;

            for v in args.iter().skip(1) {
                let Expr::Number(v) = v else {
                    panic!();
                };
                // check for valid op/not-zero
                if !(sym == "/" && *v == 0) {
                    out = func(out, *v);
                } else {
                    return Err(Error::DivisionByZero { line });
                }
            }
            Ok(Expr::Number(out))
        }
        Expr::Float(start) => {
            let func: fn(f32, f32) -> f32 = match sym {
                "+" => |a, b| a + b,
                "-" => |a, b| a - b,
                "*" => |a, b| a * b,
                "/" => |a, b| a / b,
                _ => panic!(),
            };
            let mut out = start;

            for v in args.iter().skip(1) {
                let Expr::Float(v) = v else {
                    panic!();
                };
                // check for valid op
                if !(sym == "/" && *v == 0.0) {
                    out = func(out, *v);
                } else {
                    return Err(Error::DivisionByZero { line });
                }
            }
            Ok(Expr::Float(out))
        }
        _ => panic!(),
    }
}
fn builtin_comp(func: &str, mut args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
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

fn builtin_head(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    single_qexpr_op!(args, func, line, |ls: Vec<Expr>| Ok(Expr::Qexpr(vec![ls.into_iter().next().unwrap()])))
}

fn builtin_last(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    single_qexpr_op!(args, func, line, |ls: Vec<Expr>| Ok(ls.into_iter().last().unwrap()))
}

fn builtin_tail(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    single_qexpr_op!(args, func, line, |ls: Vec<Expr>| Ok(Expr::Qexpr(ls.into_iter().skip(1).collect())))
}

fn builtin_list(_func: &str, args: Vec<Expr>, _line: usize) -> Result<Expr, Error> {
    Ok(Expr::Qexpr(args))
}

fn builtin_join(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    let mut out = Vec::new();
    for q in args {
        out.extend(q.into_qexpr(func, line)?);
    }
    Ok(Expr::Qexpr(out))
}

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

fn builtin_var(e: Env, func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
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
fn builtin_lambda(func: &str, e: Env, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
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

fn builtin_print(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    let a = args.into_iter().next().unwrap();
    println!("{a}");
    Ok(Expr::Sexpr(Vec::new()))
}

fn builtin_range(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
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

    Ok(Expr::Qexpr((0..rng).map(Expr::Number).collect()))
}

#[inline(always)]
fn builtin_if(func: &str, e: Env, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
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

fn builtin_ord(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
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

fn builtin_load(func: &str, e: Env, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    single_string_op!(args, func, line, |path: String, _func: &str, line: usize| {
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
    })
}

fn builtin_read(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    single_string_op!(args, func, line, |path: String, _func: &str, line: usize| {
        let contents = fs::read_to_string(&path).map_err(|err| Error::IoError {
            msg: format!("Failed to load file '{}': {}", path, err),
            line,
        })?;
        Ok(Expr::String(contents))
    })
}

fn builtin_chars(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    let s = args[0].clone().into_string(func, line)?;
    Ok(Expr::Qexpr(s.chars().map(Expr::Char).collect()))
}

fn builtin_int(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    match args[0].clone().into_string(func, line)?.parse::<i32>() {
        Ok(v) => Ok(Expr::Number(v)),
        Err(_) => Err(Error::ParseError {
            msg: "couldn't parse int from string".to_string(),
            line,
        }),
    }
}
fn builtin_sort(func: &str, mut args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;

    let mut nums: Vec<i32> = args
        .remove(0)
        .into_qexpr(func, line)?
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
    Ok(Expr::Qexpr(nums.iter().map(|i| Expr::Number(*i)).collect()))
}

#[inline(always)]
pub fn eval_builtin(env: Env, sym: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    if matches!(sym, "+" | "-" | "*" | "/") {
        return _builtin_op(sym, args, line);
    }

    if matches!(sym, "==" | "!=") {
        return builtin_comp(sym, args, line);
    }

    if matches!(sym, ">" | "<" | ">=" | "<=") {
        return builtin_ord(sym, args, line);
    }

    // handle builtin functions
    match sym {
        "head" => builtin_head(sym, args, line),
        "last" => builtin_last(sym, args, line),
        "tail" => builtin_tail(sym, args, line),
        "list" => builtin_list(sym, args, line),
        "join" => builtin_join(sym, args, line),
        "eval" => builtin_eval(sym, env, args, line),
        "def" => builtin_var(env, "def", args, line),
        "=" => builtin_var(env, "=", args, line),
        "\\" => builtin_lambda(sym, env, args, line),
        "print" => builtin_print(sym, args, line),
        "range" => builtin_range(sym, args, line),
        "if" => builtin_if(sym, env, args, line),
        "load" => builtin_load(sym, env, args, line),
        "read" => builtin_read(sym, args, line),
        "chars" => builtin_chars(sym, args, line),
        "int" => builtin_int(sym, args, line),
        "sort" => builtin_sort(sym, args, line),
        _ => panic!(),
    }
}

pub fn setup_builtins() -> Env {
    let env = Env::new();

    // Define a list of all builtin operator names
    let builtins = [
        "+", "-", "*", "/", "head", "last", "tail", "list", "join", "range", "eval", "if", "print",
        "load", "read", "==", "!=", ">", ">=", "<", "<=", "=", "def", "\\", "chars", "int", "sort",
    ];

    for op in builtins {
        env.insert(op.to_string(), Expr::Builtin(op.to_string()));
    }

    env
}
