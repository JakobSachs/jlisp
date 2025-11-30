use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
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

    pub fn remove(&self, key: &str) {
        ENV_STORAGE.with(|storage| {
            storage.borrow_mut()[self.0].map.remove(key);
        })
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
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
    Char(char),
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
    pub fn as_str(&self) -> String {
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

    #[inline(always)]
    pub fn into_sexpr(self, op: &str, line: usize) -> Result<Vec<Expr>, Error> {
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

    #[inline(always)]
    pub fn into_number(self, op: &str, line: usize) -> Result<i32, Error> {
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

    #[inline(always)]
    pub fn into_qexpr(self, op: &str, line: usize) -> Result<Vec<Expr>, Error> {
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
    #[inline(always)]
    pub fn into_symbol(self, op: &str, line: usize) -> Result<String, Error> {
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
    #[inline(always)]
    pub fn into_string(self, op: &str, line: usize) -> Result<String, Error> {
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
#[inline(always)]
pub fn expect_arity(func: &str, args: &[Expr], n: usize, line: usize) -> Result<(), Error> {
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
#[inline(always)]
pub fn expect_nonempty(func: &str, args: &[Expr], line: usize) -> Result<(), Error> {
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
            Expr::Char(v) => write!(f, "'{}'", v.escape_default()),
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

#[inline(always)]
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
        let res = crate::builtin::builtin_eval(func, e, vec![*body.clone()], line);
        // TODO: I think we can do some garbage collectoin of the Envs right here
        res
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
    #[inline(always)] // a load bearing inline
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
                let mut cells = sexpr
                    .into_iter()
                    .map(|e| e.clone().eval(env, line))
                    .collect::<Result<Vec<_>, _>>()?;

                if cells.is_empty() {
                    return Ok(Expr::Sexpr(Vec::new()));
                } else if cells.len() == 1 {
                    return Ok(cells.pop().unwrap());
                }

                let op = cells.remove(0);

                match op {
                    Expr::Builtin(sym) => {
                        crate::builtin::eval_builtin(env, sym.as_str(), cells, line)
                    }
                    Expr::Lambda { .. } => _eval_lambda(env, op, cells, line),
                    _ => Err(Error::MissingOperator { line }),
                }
            }
        }
    }
}
