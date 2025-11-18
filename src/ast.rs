use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::mem;
use std::vec::Vec;

use slotmap::SlotMap;

pub type EnvId = slotmap::DefaultKey;

#[derive(Debug, Clone)]
struct EnvData {
    pub map: HashMap<String, Expr>,
    pub parent: Option<EnvId>,
}

thread_local! {
    static ENV_STORAGE: RefCell<SlotMap<EnvId, EnvData>> = RefCell::new(SlotMap::with_key());
}

/// Environment handle - cheap to copy and pass around
#[derive(Debug, Clone, Copy)]
pub struct Env(EnvId);

impl Env {
    /// Create a new root environment
    pub fn new() -> Self {
        ENV_STORAGE.with(|storage| {
            let id = storage.borrow_mut().insert(EnvData {
                map: HashMap::new(),
                parent: None,
            });
            Env(id)
        })
    }

    /// Create a child environment with the given parent
    pub fn child(parent: Env) -> Self {
        ENV_STORAGE.with(|storage| {
            let id = storage.borrow_mut().insert(EnvData {
                map: HashMap::new(),
                parent: Some(parent.0),
            });
            Env(id)
        })
    }

    /// Get a value from this environment or any parent
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

    /// Insert a value into this environment (not parents)
    pub fn insert(&self, key: String, val: Expr) {
        ENV_STORAGE.with(|storage| {
            storage.borrow_mut()[self.0].map.insert(key, val);
        })
    }

    /// Get the root environment by walking up the parent chain
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
        env: Env, // now just Env, not Rc<Env>
        formals: Vec<Expr>,
        body: Box<Expr>,
    },
    Symbol(String),
    String(String),
    Comment(String),
    Sexpr(Vec<Expr>),
    Qexpr(Vec<Expr>),
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

#[derive(Debug)]
pub enum Error {
    UndefinedSymbol(String),
    DivisionByZero,
    IncompatibleType,
    InconsistentTypes,
    MissingOperator,
    WrongAmountOfArgs,
}

fn _builtin_op(sym: String, a: Expr) -> Result<Expr, Error> {
    let Expr::Sexpr(args) = a else {
        panic!("expected s-expr for _builtin_op");
    };

    //check type of first member is valid
    if !matches!(args[0], Expr::Number(_) | Expr::Float(_) | Expr::Char(_)) {
        return Err(Error::IncompatibleType);
    }

    // check all for same type (not content)
    if args.len() > 1
        && !args.first().map_or(true, |first| {
            let first_type = mem::discriminant(first);
            args.iter().all(|x| mem::discriminant(x) == first_type)
        })
    {
        dbg!(args);
        return Err(Error::InconsistentTypes);
    }

    if sym == "-" && args.len() == 1 {
        match args[0] {
            Expr::Number(n) => return Ok(Expr::Number(-n)),
            Expr::Float(f) => return Ok(Expr::Float(-f)),
            _ => return Err(Error::IncompatibleType),
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
                    return Err(Error::DivisionByZero);
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
                    return Err(Error::DivisionByZero);
                }
            }
            return Ok(Expr::Float(out));
        }
        _ => panic!(),
    }
}

fn builtin_head(a: Expr) -> Result<Expr, Error> {
    let Expr::Sexpr(args) = a else {
        return Err(Error::IncompatibleType);
    };
    if args.len() != 1 {
        return Err(Error::WrongAmountOfArgs);
    }
    let Expr::Qexpr(ls) = args[0].clone() else {
        return Err(Error::IncompatibleType);
    };

    if ls.is_empty() {
        return Err(Error::WrongAmountOfArgs);
    }

    Ok(ls[0].clone())
}

fn builtin_last(a: Expr) -> Result<Expr, Error> {
    let Expr::Sexpr(args) = a else {
        return Err(Error::IncompatibleType);
    };
    if args.len() != 1 {
        return Err(Error::WrongAmountOfArgs);
    }
    let Expr::Qexpr(ls) = args[0].clone() else {
        return Err(Error::IncompatibleType);
    };

    if ls.is_empty() {
        return Err(Error::WrongAmountOfArgs);
    }

    Ok(ls[ls.len() - 1].clone())
}

fn builtin_tail(a: Expr) -> Result<Expr, Error> {
    let Expr::Sexpr(args) = a else {
        return Err(Error::IncompatibleType);
    };
    if args.len() != 1 {
        return Err(Error::WrongAmountOfArgs);
    }
    let Expr::Qexpr(ls) = args[0].clone() else {
        return Err(Error::IncompatibleType);
    };

    if ls.is_empty() {
        return Err(Error::WrongAmountOfArgs);
    }

    Ok(Expr::Sexpr(ls[1..].to_vec()))
}

fn builtin_list(a: Expr) -> Result<Expr, Error> {
    let Expr::Sexpr(args) = a else {
        return Err(Error::IncompatibleType);
    };
    Ok(Expr::Qexpr(args))
}

fn builtin_join(a: Expr) -> Result<Expr, Error> {
    let Expr::Sexpr(args) = a else {
        return Err(Error::IncompatibleType);
    };

    let mut out: Vec<Expr> = Vec::new();
    for c in args {
        let Expr::Qexpr(l) = c else {
            return Err(Error::IncompatibleType);
        };
        out.extend(l);
    }

    Ok(Expr::Qexpr(out))
}

fn builtin_eval(e: Env, a: Expr) -> Result<Expr, Error> {
    let Expr::Sexpr(args) = a else {
        return Err(Error::IncompatibleType);
    };
    if args.len() != 1 {
        return Err(Error::WrongAmountOfArgs);
    }
    let Expr::Qexpr(ls) = args[0].clone() else {
        return Err(Error::IncompatibleType);
    };

    Expr::Sexpr(ls).eval(e)
}

fn builtin_var(e: Env, func: String, a: Expr) -> Result<Expr, Error> {
    let Expr::Sexpr(args) = a else {
        return Err(Error::IncompatibleType);
    };

    // check symbols
    let Expr::Qexpr(ref symbols) = args[0] else {
        return Err(Error::IncompatibleType);
    };

    for s in symbols.iter() {
        let Expr::Symbol(_) = s else {
            return Err(Error::IncompatibleType);
        };
    }

    // match lenght
    if (args.len() - 1) != symbols.len() {
        return Err(Error::WrongAmountOfArgs);
    }

    for (sy, ar) in symbols.iter().zip(args.iter().skip(1)) {
        let Expr::Symbol(sy) = sy else {
            panic!();
        };
        match func.as_str() {
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
fn builtin_lambda(e: Env, a: Expr) -> Result<Expr, Error> {
    let Expr::Sexpr(args) = a else {
        return Err(Error::IncompatibleType);
    };

    if args.len() != 2 {
        return Err(Error::WrongAmountOfArgs);
    }

    // check arg types
    let Expr::Qexpr(formals) = &args[0] else {
        return Err(Error::IncompatibleType);
    };
    let Expr::Qexpr(_) = args[1] else {
        return Err(Error::IncompatibleType);
    };

    for f in formals.iter() {
        let Expr::Symbol(_) = f else {
            return Err(Error::IncompatibleType);
        };
    }

    // Create a new environment for the lambda that captures the current environment
    let lambda_env = Env::child(e);

    Ok(Expr::Lambda {
        env: lambda_env,
        formals: formals.clone(),
        body: Box::new(args[1].clone()),
    })
}

fn _eval_builtin(env: Env, sym: String, a: Expr) -> Result<Expr, Error> {
    if matches!(sym.as_str(), "+" | "-" | "*" | "/") {
        return _builtin_op(sym, a);
    }
    // handle builtin functions
    match sym.as_str() {
        "head" => return builtin_head(a),
        "last" => return builtin_last(a),
        "tail" => return builtin_tail(a),
        "list" => return builtin_list(a),
        "join" => return builtin_join(a),
        "eval" => return builtin_eval(env, a),
        "def" => return builtin_var(env, "def".to_string(), a),
        "=" => return builtin_var(env, "=".to_string(), a),
        "\\" => return builtin_lambda(env, a),
        _ => panic!(),
    };
}

fn _eval_lambda(_env: Env, op: Expr, a: Expr) -> Result<Expr, Error> {
    let Expr::Lambda {
        env: e,
        mut formals,
        body,
    } = op
    else {
        panic!();
    };
    let Expr::Sexpr(args) = a else {
        return Err(Error::IncompatibleType);
    };

    // Bind arguments to the lambda's environment
    for val in args {
        if formals.is_empty() {
            return Err(Error::WrongAmountOfArgs);
        }
        let Expr::Symbol(sym) = formals.remove(0) else {
            return Err(Error::IncompatibleType);
        };
        e.insert(sym.clone(), val.clone());
    }

    if formals.is_empty() {
        // All args bound, evaluate the body
        return builtin_eval(e, Expr::Sexpr(vec![*body.clone()]));
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
    pub fn eval(self, env: Env) -> Result<Expr, Error> {
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
                None => Err(Error::UndefinedSymbol(sym)),
            },
            Expr::Sexpr(sexpr) => {
                let cells = sexpr
                    .iter()
                    .map(|e| e.clone().eval(env))
                    .collect::<Result<Vec<_>, _>>()?;

                if cells.len() == 0 {
                    return Ok(Expr::Sexpr(Vec::new()));
                } else if cells.len() == 1 {
                    return Ok(cells[0].clone());
                }

                let (op, s_children) = cells.split_at(1);
                assert!(op.len() == 1);
                let children = Expr::Sexpr(s_children.to_vec());

                match &op[0] {
                    Expr::Builtin(sym) => _eval_builtin(env, (&sym).to_string(), children),
                    Expr::Lambda { .. } => _eval_lambda(env, (&op[0]).clone(), children),
                    _ => Err(Error::MissingOperator),
                }
            }
        }
    }
}
