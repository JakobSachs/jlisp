use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::mem;
use std::vec::Vec;
use std::backtrace::Backtrace;

use slotmap::SlotMap;
use thiserror::Error;

// i got this to work, but honestly dont 100% understand how this slotmap works
pub type EnvId = slotmap::DefaultKey;

#[derive(Debug, Clone)]
struct EnvData {
    pub map: HashMap<String, Expr>,
    pub parent: Option<EnvId>,
}

thread_local! {
    static ENV_STORAGE: RefCell<SlotMap<EnvId, EnvData>> = RefCell::new(SlotMap::with_key());
}

#[derive(Debug, Clone, Copy)]
pub struct Env(EnvId);

impl Env {
    pub fn new() -> Self {
        ENV_STORAGE.with(|storage| {
            let id = storage.borrow_mut().insert(EnvData {
                map: HashMap::new(),
                parent: None,
            });
            Env(id)
        })
    }

    pub fn child(parent: Env) -> Self {
        ENV_STORAGE.with(|storage| {
            let id = storage.borrow_mut().insert(EnvData {
                map: HashMap::new(),
                parent: Some(parent.0),
            });
            Env(id)
        })
    }

    pub fn get(&self, key: &str) -> Option<Expr> {
        ENV_STORAGE.with(|storage| {
            let storage = storage.borrow();
            let mut current = Some(self.0);
            while let Some(id) = current {
                let data = &storage[id];
                if let Some(v) = data.map.get(key) {
                    return Some(v.clone());
                }
                current = data.parent;
            }
            None
        })
    }

    pub fn insert(&self, key: String, val: Expr) {
        ENV_STORAGE.with(|storage| {
            storage.borrow_mut()[self.0].map.insert(key, val);
        })
    }

    pub fn root(&self) -> Env {
        ENV_STORAGE.with(|storage| {
            let storage = storage.borrow();
            let mut current = self.0;
            loop {
                let data = &storage[current];
                match data.parent {
                    Some(p) => current = p,
                    None => return Env(current),
                }
            }
        })
    }
}

#[derive(Debug)]
pub struct JLisp {
    pub exprs: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i32),
    Float(f32),
    Char(u8),
    Builtin(String),
    Lambda {
        env: Env,
        formals: Vec<Expr>,
        body: Box<Expr>,
    },
    Symbol(String),
    String(String),
    Comment(String),
    Sexpr(Vec<Expr>),
    Qexpr(Vec<Expr>),
}

//helpers
impl Expr {
    fn as_str(&self) -> String {
        match self {
            Expr::Number(_) => "Number".to_string(),
            Expr::Float(_) => "Float".to_string(),
            Expr::Char(_) => "Char".to_string(),
            Expr::Builtin(_) => "Builtin".to_string(),
            Expr::Lambda { .. } => "Lambda".to_string(),
            Expr::Symbol(s) => format!("Symbol: {s}"),
            Expr::String(_) => "String".to_string(),
            Expr::Comment(_) => "Comment".to_string(),
            Expr::Sexpr(_) => "Sexpr".to_string(),
            Expr::Qexpr(_) => "Qexpr".to_string(),
        }
    }

    fn _into_sexpr(self, op: &str, line: usize) -> Result<Vec<Expr>, Error> {
        if let Expr::Sexpr(v) = self {
            Ok(v)
        } else {
            Err(Error::IncompatibleType {
                op: op.to_string(),
                expected: "Sexpr".to_string(),
                received: self.as_str(),
                line: line,
            })
        }
    }
    fn into_qexpr(self, op: &str, line: usize) -> Result<Vec<Expr>, Error> {
        if let Expr::Qexpr(v) = self {
            Ok(v)
        } else {
            Err(Error::IncompatibleType {
                op: op.to_string(),
                expected: "Qexpr".to_string(),
                received: self.as_str(),
                line: line,
            })
        }
    }
    fn into_symbol(&self, op: &str, line: usize) -> Result<String, Error> {
        if let Expr::Symbol(s) = self {
            Ok(String::from(s))
        } else {
            Err(Error::IncompatibleType {
                op: op.to_string(),
                expected: "Symbol".to_string(),
                received: self.as_str(),
                line: line,
            })
        }
    }
}
// arity helper
fn expect_arity(func: &str, args: &[Expr], n: usize, line: usize) -> Result<(), Error> {
    if args.len() == n {
        Ok(())
    } else {
        Err(Error::WrongAmountOfArgs {
            func: func.to_string(),
            expected: n,
            received: args.len(),
            line,
        })
    }
}
fn expect_nonempty(func: &str, args: &[Expr], line: usize) -> Result<(), Error> {
    if args.len() == 0 {
        return Err(Error::WrongAmountOfArgs {
            func: func.to_string(),
            expected: 1,
            received: args.len(),
            line,
        });
    }
    Ok(())
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Number(v) => write!(f, "{}", v),
            Expr::Float(v) => write!(f, "{}", v),
            Expr::Symbol(v) => write!(f, "{}", v),
            Expr::Char(v) => write!(f, "'{}'", v),
            Expr::String(v) => write!(f, "\"{}\"", v),
            Expr::Lambda {
                env: _,
                formals,
                body,
            } => {
                write!(f, "(\\")?;
                for (i, form) in formals.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", form)?;
                }
                write!(f, "{})", body)
            }
            Expr::Builtin(_) => write!(f, "<builtin>"),
            Expr::Sexpr(vals) => {
                write!(f, "(")?;
                for (i, v) in vals.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, ")")
            }
            Expr::Qexpr(vals) => {
                write!(f, "{{")?;
                for (i, v) in vals.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "}}")
            }
            Expr::Comment(_) => write!(f, "()"),
        }
    }
}



#[derive(Debug, Error)]
pub enum Error {
    #[error("undefined symbol '{sym}' at line {line}")]
    UndefinedSymbol { sym: String, line: usize },
    #[error("Tried to divide by zero at line {line}")]
    DivisionByZero { line: usize },
    #[error("type error in '{op}' expected {expected}, got {received} at line {line}")]
    IncompatibleType {
        op: String,
        expected: String,
        received: String,
        line: usize,
    },
    #[error("mixed types in '{op}' at line {line}")]
    InconsistentTypes { op: String, line: usize },
    #[error("missing operator at line {line}")]
    MissingOperator { line: usize },
    #[error(
        "wrong amount of args to func '{func}', expected {expected} but got {received} at line {line}"
    )]
    WrongAmountOfArgs {
        func: String,
        expected: usize,
        received: usize,
        line: usize,
    },
}

fn _builtin_op(sym: String, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    //check type of first member is valid
    if !matches!(args[0], Expr::Number(_) | Expr::Float(_) | Expr::Char(_)) {
        return Err(Error::IncompatibleType {
            op: sym,
            expected: "Number,Float,Char".to_string(),
            received: args[0].as_str(),
            line,
        });
    }

    // check all for same type (not content)
    if args.len() > 1
        && !args.first().map_or(true, |first| {
            let first_type = mem::discriminant(first);
            args.iter().all(|x| mem::discriminant(x) == first_type)
        })
    {
        return Err(Error::InconsistentTypes { op: sym, line });
    }

    if sym == "-" && args.len() == 1 {
        match args[0] {
            Expr::Number(n) => return Ok(Expr::Number(-n)),
            Expr::Float(f) => return Ok(Expr::Float(-f)),
            _ => {
                return Err(Error::IncompatibleType {
                    op: sym,
                    expected: "Number,Float".to_string(),
                    received: args[0].as_str(),
                    line,
                });
            }
        }
    }

    match args[0] {
        Expr::Number(start) => {
            let func: fn(i32, i32) -> i32 = match sym.as_str() {
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
                // check for valid op
                if sym.as_str() != "/" || (sym.as_str() == "/" && *v != 0) {
                    out = func(out, *v);
                } else {
                    return Err(Error::DivisionByZero { line });
                }
            }
            return Ok(Expr::Number(out));
        }
        Expr::Float(start) => {
            let func: fn(f32, f32) -> f32 = match sym.as_str() {
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
                if sym.as_str() != "/" || (sym.as_str() == "/" && *v != 0.0) {
                    out = func(out, *v);
                } else {
                    return Err(Error::DivisionByZero { line });
                }
            }
            return Ok(Expr::Float(out));
        }
        _ => panic!(),
    }
}

fn builtin_head(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;

    let ls = args
        .into_iter()
        .next()
        .unwrap()
        .into_qexpr(func, line)?;

    expect_nonempty(func, &ls, line)?;
    Ok(Expr::Qexpr(vec![ls.into_iter().next().unwrap()]))
}

fn builtin_last(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    let ls = args
        .into_iter()
        .next()
        .unwrap()
        .into_qexpr(func, line)?;
    expect_nonempty(func, &ls, line)?;
    Ok(ls.into_iter().last().unwrap())
}

fn builtin_tail(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    let ls = args
        .into_iter()
        .next()
        .unwrap()
        .into_qexpr(func, line)?;
    expect_nonempty(func, &ls, line)?;
    Ok(Expr::Qexpr(ls.into_iter().skip(1).collect()))
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

fn builtin_eval(func: &str, e: Env, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    let ls = args
        .into_iter()
        .next()
        .unwrap()
        .into_qexpr(func, line)?;
    Expr::Sexpr(ls).eval(e, line)
}

fn builtin_var(e: Env, func: &str, args: Vec<Expr>,line: usize) -> Result<Expr, Error> {
    expect_nonempty(func,&args,line)?;

    // check symbols
    let symbols = args[0].clone().into_qexpr(func,line)?;
    for s in symbols.iter() {
        s.into_symbol(func,line)?;
    }

    // match lenght
    expect_arity(func,&args, symbols.len() + 1,line)?;
    for (sy, ar) in symbols.iter().zip(args.iter().skip(1)) {
        let sy = sy.into_symbol(func,line)?;
        match func {
            "def" => {
                // Insert into root environment
                let root = e.root();
                root.insert(sy.clone(), ar.clone());
            }
            "=" => {
                // Insert into current environment
                e.insert(sy.clone(), ar.clone());
            }
            _ => panic!(),
        }
    }

    Ok(Expr::Sexpr(Vec::new()))
}
fn builtin_lambda(func: &str,e: Env, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func,&args, 2,line)?;

    // check arg types
    let formals = args[0].clone().into_qexpr(func,line)?;
    let _ = args[1].clone().into_qexpr(func,line)?; // rest 

    for f in formals.iter() {
        let _ = f.into_symbol(func,line)?;
    }

    // Create a new environment for the lambda that captures the current environment
    let lambda_env = Env::child(e);

    Ok(Expr::Lambda {
        env: lambda_env,
        formals: formals.to_vec(),
        body: Box::new(args[1].clone()),
    })
}

fn _eval_builtin(env: Env, sym: &str, args: Vec<Expr>, line:usize) -> Result<Expr, Error> {
    if matches!(sym, "+" | "-" | "*" | "/") {
        return _builtin_op(sym.to_string(), args,line);
    }
    // handle builtin functions
    match sym {
        "head" => return builtin_head(sym,args,line),
        "last" => return builtin_last(sym,args,line),
        "tail" => return builtin_tail(sym,args,line),
        "list" => return builtin_list(sym,args,line),
        "join" => return builtin_join(sym,args,line),
        "eval" => return builtin_eval(sym,env, args,line),
        "def" => return builtin_var(env, "def", args,line),
        "=" => return builtin_var(env, "=", args,line),
        "\\" => return builtin_lambda(sym,env, args,line),
        _ => panic!(),
    };
}

fn _eval_lambda(_env: Env, op: Expr, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    let func = "\\";
    let Expr::Lambda {
        env: e,
        mut formals,
        body,
    } = op
    else {
        panic!();
    };

    // Bind arguments to the lambda's environment
    let mut i = 0;
    while i < args.len() {
        let val = args.get(i).unwrap();
        expect_nonempty(func,&formals,line)?;
        let sym = formals.remove(0).into_symbol(func,line)?;

        if sym == "&" {
            // require one more symbol
            expect_nonempty(func,&formals,line)?;
            let rest_sym = formals.remove(0).into_symbol(func,line)?;

            // take rest of args and quit loop
            let rest_args: Vec<_> = args.iter().skip(i + 1).cloned().collect();
            e.insert(rest_sym.clone(), Expr::Qexpr(rest_args));
            break;
        }

        e.insert(sym.clone(), val.clone());
        i += 1;
    }

    // handle case where the & arg needs to bind to an empty list
    if !formals.is_empty() {
        let list_sym = formals.get(0).unwrap().into_symbol(func,line)?;
        if list_sym == "&" {
            expect_arity(func,&formals, 2,line)?;
            let bind_sym = formals.get(1).unwrap().into_symbol(func, line)?;
            e.insert(bind_sym.clone(), Expr::Qexpr(Vec::new()));
            formals.clear();
        }
    }

    if formals.is_empty() {
        // All args bound, evaluate the body
        return builtin_eval(func,e, vec![*body.clone()],line);
    } else {
        // Partial application - return partial lambda
        return Ok(Expr::Lambda {
            env: e,
            formals,
            body,
        });
    }
}

impl Expr {
    pub fn eval(self, env: Env, line: usize) -> Result<Expr, Error> {
        match self {
            Expr::Number(_)
            | Expr::Float(_)
            | Expr::Char(_)
            | Expr::String(_)
            | Expr::Qexpr(_)
            | Expr::Builtin(_)
            | Expr::Lambda { .. } => {
                return Ok(self);
            }
            Expr::Comment(_) => return Ok(Expr::Sexpr(Vec::new())),
            Expr::Symbol(sym) => match env.get(&sym) {
                Some(v) => Ok(v),
                None => Err(Error::UndefinedSymbol { sym, line }),
            },
            Expr::Sexpr(sexpr) => {
                let cells = sexpr
                    .iter()
                    .map(|e| e.clone().eval(env, line))
                    .collect::<Result<Vec<_>, _>>()?;

                if cells.len() == 0 {
                    return Ok(Expr::Sexpr(Vec::new()));
                } else if cells.len() == 1 {
                    return Ok(cells[0].clone());
                }

                let (op, s_children) = cells.split_at(1);
                assert!(op.len() == 1);
                let children = s_children.to_vec();

                match &op[0] {
                    Expr::Builtin(sym) => _eval_builtin(env, sym.as_str(), children,line),
                    Expr::Lambda { .. } => _eval_lambda(env, (&op[0]).clone(), children,line),
                    _ => Err(Error::MissingOperator { line }),
                }
            }
        }
    }
}
