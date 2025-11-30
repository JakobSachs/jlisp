use jlisp::ast::Expr;
use jlisp::env::Env;
use std::cell::RefCell;

thread_local! {
    static PERSISTENT_ENV: RefCell<Env> = RefCell::new(jlisp::builtin::setup_builtins());
}

pub fn eval_str(input: &str) -> Result<Expr, Box<dyn std::error::Error + '_>> {
    let env = jlisp::builtin::setup_builtins();
    let pe = jlisp::grammar::ExprParser::new();
    let pres = pe.parse(input)?;
    let res = pres.eval(env, 0)?;
    Ok(res)
}

pub fn eval_str_persistent(input: &str) -> Result<Expr, Box<dyn std::error::Error + '_>> {
    let pe = jlisp::grammar::ExprParser::new();
    let pres = pe.parse(input)?;
    let res = PERSISTENT_ENV.with(|env| pres.eval(env.borrow().clone(), 0))?;
    Ok(res)
}

pub fn reset_persistent_env() {
    PERSISTENT_ENV.with(|env| {
        *env.borrow_mut() = jlisp::builtin::setup_builtins();
    });
}
