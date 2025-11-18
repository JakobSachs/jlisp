use std::cell::RefCell;
use std::collections::HashMap;
use std::mem;
use std::rc::Rc;
use std::vec::Vec;

use slotmap::{SecondaryMap, SlotMap};

pub type EnvId = slotmap::DefaultKey;

struct EnvStorage {
    pool: SlotMap<EnvId, EnvData>,
}

impl EnvStorage {
    pub fn new() -> Self {
        Self {
            pool: SlotMap::with_key(),
        }
    }

    pub fn create(&mut self, parent: Option<Env>) -> Env {
        let id = self.pool.insert(EnvData {
            map: HashMap::new(),
            parent: parent.map(|p| p.0),
        });
        Env(id)
    }
    pub fn get(&self, mut env: Env, key: &str) -> Option<&Expr> {
        loop {
            let data = &self.pool[env.0];
            if let Some(v) = data.map.get(key) {
                return Some(v);
            }

            // walk up
            match data.parent {
                Some(p) => env = Env(p),
                None => return None,
            }
        }
    }

    pub fn insert(&mut self, env: Env, key: String, val: Expr)  {
        let _ = self.pool[env.0].map.insert(key, val);
    }
    
    pub fn set_parent(&mut self, 
        env: Env, 
        parent: Option<Env>) {
        self.pool[env.0].parent = parent.map(|p| p.0);
    }


}

#[derive(Debug, Clone)]
struct EnvData {
    pub map: HashMap<String, Expr>,
    pub parent: Option<EnvId>,
}
#[derive(Debug, Clone)]
pub struct Env(EnvId);

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

fn builtin_eval(e: Rc<Env>, a: Expr) -> Result<Expr, Error> {
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

fn builtin_var(e: Rc<Env>, func: String, a: Expr) -> Result<Expr, Error> {
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
                let top = std::iter::successors(Some(&*e), |n| n.parent.as_deref())
                    .last()
                    .unwrap();
                top.map.borrow_mut().insert(sy.clone(), ar.clone());
            }
            "=" => {
                let _ = e.map.borrow_mut().insert(sy.clone(), ar.clone());
            }
            _ => panic!(),
        }
    }

    Ok(Expr::Sexpr(Vec::new()))
}
fn builtin_lambda(e: Rc<Env>, a: Expr) -> Result<Expr, Error> {
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

    Ok(Expr::Lambda {
        env: Env::new(),
        formals: formals.clone(),
        body: Box::new(args[1].clone()),
    })
}

fn _eval_builtin(env: Rc<Env>, sym: String, a: Expr) -> Result<Expr, Error> {
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

fn _eval_lambda(env: Rc<Env>, op: Expr, a: Expr) -> Result<Expr, Error> {
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

    for val in args {
        if formals.is_empty() {
            return Err(Error::WrongAmountOfArgs);
        }
        let Expr::Symbol(sym) = formals.remove(0) else {
            return Err(Error::IncompatibleType);
        };
        env.map.borrow_mut().insert(sym.clone(), val.clone());
    }

    if formals.is_empty() {
        // unbound
        return Ok(Expr::Lambda {
            env: e,
            formals,
            body,
        });
    } else {
        // = Some(env);
        return builtin_eval(e, Expr::Sexpr(vec![*body.clone()]));
    }
}

impl Expr {
    pub fn eval(self, env: Rc<Env>) -> Result<Expr, Error> {
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
            Expr::Symbol(sym) => match env.map.borrow().get(&sym) {
                Some(v) => Ok(v.clone()),
                None => Err(Error::UndefinedSymbol(sym)),
            },
            Expr::Sexpr(sexpr) => {
                let cells = sexpr
                    .iter()
                    .map(|e| e.clone().eval(env.clone()))
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
                    Expr::Builtin(sym) => _eval_builtin(env.clone(), (&sym).to_string(), children),
                    Expr::Lambda { .. } => _eval_lambda(env.clone(), (&op[0]).clone(), children),
                    _ => Err(Error::MissingOperator),
                }
            }
        }
    }
}
