use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::mem;
use std::vec::Vec;

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

impl PartialEq for Expr {
    fn eq(&self, other: &Expr) -> bool {
        if mem::discriminant(self) != mem::discriminant(other) {
            return false;
        }

        match self {
            Expr::Number(v) => {
                let Expr::Number(vother) = other else {
                    panic!();
                };
                v == vother
            }
            Expr::Float(v) => {
                let Expr::Float(vother) = other else {
                    panic!();
                };
                v == vother
            }
            Expr::Char(v) => {
                let Expr::Char(vother) = other else {
                    panic!();
                };
                v == vother
            }
            Expr::Symbol(s) => {
                let Expr::Symbol(sother) = other else {
                    panic!();
                };
                s == sother
            }
            Expr::String(s) => {
                let Expr::String(sother) = other else {
                    panic!();
                };
                s == sother
            }
            Expr::Comment(_) => true,
            Expr::Builtin(b) => {
                let Expr::Builtin(bother) = other else {
                    panic!();
                };
                b == bother
            }
            Expr::Lambda {
                env: _,
                formals,
                body,
            } => {
                let Expr::Lambda {
                    env: _,
                    formals: other_formals,
                    body: other_body,
                } = other
                else {
                    panic!();
                };

                (formals == other_formals) && (body == other_body)
            }
            Expr::Sexpr(cells) => {
                let Expr::Sexpr(cells_other) = other else {
                    panic!();
                };
                if cells.len() != cells_other.len() {
                    return false;
                }
                for (c, o_c) in cells.iter().zip(cells_other.iter()) {
                    if c != o_c {
                        return false;
                    }
                }
                true
            }
            Expr::Qexpr(cells) => {
                let Expr::Qexpr(cells_other) = other else {
                    panic!();
                };
                if cells.len() != cells_other.len() {
                    return false;
                }

                for (c, o_c) in cells.iter().zip(cells_other.iter()) {
                    if c != o_c {
                        return false;
                    }
                }
                true
            }
        }
    }
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
                line,
            })
        }
    }
    fn into_number(self, op: &str, line: usize) -> Result<i32, Error> {
        if let Expr::Number(i) = self {
            Ok(i)
        } else {
            Err(Error::IncompatibleType {
                op: op.to_string(),
                expected: "Number".to_string(),
                received: self.as_str(),
                line,
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
                line,
            })
        }
    }
    fn into_symbol(self, op: &str, line: usize) -> Result<String, Error> {
        if let Expr::Symbol(s) = self {
            Ok(s)
        } else {
            Err(Error::IncompatibleType {
                op: op.to_string(),
                expected: "Symbol".to_string(),
                received: self.as_str(),
                line,
            })
        }
    }
    fn into_string(self, op: &str, line: usize) -> Result<String, Error> {
        if let Expr::String(s) = self {
            Ok(s)
        } else {
            Err(Error::IncompatibleType {
                op: op.to_string(),
                expected: "String".to_string(),
                received: self.as_str(),
                line,
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
    if args.is_empty() {
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
    #[error("IO error: {msg} at line {line}")]
    IoError { msg: String, line: usize },
    #[error("Parse error: {msg} at line {line}")]
    ParseError { msg: String, line: usize },
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
            Ok(Expr::Number(out))
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
            Ok(Expr::Float(out))
        }
        _ => panic!(),
    }
}
fn builtin_comp(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 2, line)?;
    let a = args[0].clone();
    let b = args[1].clone();

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
    expect_arity(func, &args, 1, line)?;

    let ls = args.into_iter().next().unwrap().into_qexpr(func, line)?;

    expect_nonempty(func, &ls, line)?;
    Ok(Expr::Qexpr(vec![ls.into_iter().next().unwrap()]))
}

fn builtin_last(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    let ls = args.into_iter().next().unwrap().into_qexpr(func, line)?;
    expect_nonempty(func, &ls, line)?;
    Ok(ls.into_iter().last().unwrap())
}

fn builtin_tail(func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    let ls = args.into_iter().next().unwrap().into_qexpr(func, line)?;
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
    let ls = args.into_iter().next().unwrap().into_qexpr(func, line)?;
    Expr::Sexpr(ls).eval(e, line)
}

fn builtin_var(e: Env, func: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_nonempty(func, &args, line)?;

    // check symbols
    let symbols = args[0].clone().into_qexpr(func, line)?;
    for s in symbols.iter() {
        s.clone().into_symbol(func, line)?;
    }

    // match lenght
    expect_arity(func, &args, symbols.len() + 1, line)?;
    for (sy, ar) in symbols.iter().zip(args.iter().skip(1)) {
        let sy = sy.clone().into_symbol(func, line)?;
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
fn builtin_lambda(func: &str, e: Env, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 2, line)?;

    // check arg types
    let formals = args[0].clone().into_qexpr(func, line)?;
    let _ = args[1].clone().into_qexpr(func, line)?; // rest 

    for f in formals.iter() {
        let _ = f.clone().into_symbol(func, line)?;
    }

    // create a new environment for the lambda that captures the current environment
    let lambda_env = Env::child(e);

    Ok(Expr::Lambda {
        env: lambda_env,
        formals: formals.to_vec(),
        body: Box::new(args[1].clone()),
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

    Ok(Expr::Qexpr(
        (0..rng).map(|n| Expr::Number(n as i32)).collect(),
    ))
}

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

fn builtin_load(func: &str, e: Env, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    expect_arity(func, &args, 1, line)?;
    let path = args[0].clone().into_string(func, line)?;
    dbg!(&path);

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

fn _eval_builtin(env: Env, sym: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    if matches!(sym, "+" | "-" | "*" | "/") {
        return _builtin_op(sym.to_string(), args, line);
    }

    if matches!(sym, "==" | "!=") {
        return builtin_comp(sym, args, line);
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
        _ => panic!(),
    }
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
        expect_nonempty(func, &formals, line)?;
        let sym = formals.remove(0).into_symbol(func, line)?;

        if sym == "&" {
            // require one more symbol
            expect_nonempty(func, &formals, line)?;
            let rest_sym = formals.remove(0).into_symbol(func, line)?;

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
        let list_sym = formals.first().unwrap().clone().into_symbol(func, line)?;
        if list_sym == "&" {
            expect_arity(func, &formals, 2, line)?;
            let bind_sym = formals.get(1).unwrap().clone().into_symbol(func, line)?;
            e.insert(bind_sym.clone(), Expr::Qexpr(Vec::new()));
            formals.clear();
        }
    }

    if formals.is_empty() {
        // All args bound, evaluate the body
        builtin_eval(func, e, vec![*body.clone()], line)
    } else {
        // Partial application - return partial lambda
        Ok(Expr::Lambda {
            env: e,
            formals,
            body,
        })
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
            | Expr::Lambda { .. } => Ok(self),
            Expr::Comment(_) => Ok(Expr::Sexpr(Vec::new())),
            Expr::Symbol(sym) => match env.get(&sym) {
                Some(v) => Ok(v),
                None => Err(Error::UndefinedSymbol { sym, line }),
            },
            Expr::Sexpr(sexpr) => {
                let cells = sexpr
                    .iter()
                    .map(|e| e.clone().eval(env, line))
                    .collect::<Result<Vec<_>, _>>()?;

                if cells.is_empty() {
                    return Ok(Expr::Sexpr(Vec::new()));
                } else if cells.len() == 1 {
                    return Ok(cells[0].clone());
                }

                let (op, s_children) = cells.split_at(1);
                assert!(op.len() == 1);
                let children = s_children.to_vec();

                match &op[0] {
                    Expr::Builtin(sym) => _eval_builtin(env, sym.as_str(), children, line),
                    Expr::Lambda { .. } => _eval_lambda(env, op[0].clone(), children, line),
                    _ => Err(Error::MissingOperator { line }),
                }
            }
        }
    }
}
