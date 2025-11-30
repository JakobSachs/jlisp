mod arithmetic;
mod collections;
mod comparison;
mod core;
mod helpers;
mod io;
mod macros;
mod math;
mod strings;

use crate::ast::{Error, Expr};
use crate::env::Env;

// Re-export all the builtin functions
use arithmetic::builtin_op;
use collections::{
    builtin_head, builtin_join, builtin_last, builtin_len, builtin_list, builtin_split,
    builtin_tail,
};
use comparison::{builtin_comp, builtin_logic, builtin_ord};
pub use core::{builtin_eval, builtin_if, builtin_lambda, builtin_var};
use helpers::{builtin_print, builtin_range, builtin_sort};
use io::{builtin_load, builtin_read};
use math::{
    builtin_abs, builtin_ceil, builtin_cos, builtin_exp, builtin_floor, builtin_log, builtin_max,
    builtin_min, builtin_round, builtin_sin, builtin_sqrt, builtin_tan, builtin_truncate,
};
use strings::{builtin_chars, builtin_int, builtin_str_sub};

#[inline(always)]
pub fn eval_builtin(env: Env, sym: &str, args: Vec<Expr>, line: usize) -> Result<Expr, Error> {
    if matches!(
        sym,
        "+" | "-" | "*" | "/" | "%" | "**" | "&" | "|" | "^" | "<<" | ">>"
    ) {
        return builtin_op(sym, args, line);
    }

    if matches!(sym, "==" | "!=") {
        return builtin_comp(sym, args, line);
    }

    if matches!(sym, ">" | "<" | ">=" | "<=") {
        return builtin_ord(sym, args, line);
    }

    if matches!(sym, "and" | "or" | "not") {
        return builtin_logic(sym, args, line);
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
        "len" => builtin_len(sym, args, line),
        "str-sub" => builtin_str_sub(sym, args, line),
        "split" => builtin_split(sym, args, line),
        "sqrt" => builtin_sqrt(sym, args, line),
        "abs" => builtin_abs(sym, args, line),
        "min" => builtin_min(sym, args, line),
        "max" => builtin_max(sym, args, line),
        "floor" => builtin_floor(sym, args, line),
        "ceil" => builtin_ceil(sym, args, line),
        "round" => builtin_round(sym, args, line),
        "sin" => builtin_sin(sym, args, line),
        "cos" => builtin_cos(sym, args, line),
        "tan" => builtin_tan(sym, args, line),
        "log" => builtin_log(sym, args, line),
        "exp" => builtin_exp(sym, args, line),
        "truncate" => builtin_truncate(sym, args, line),
        _ => panic!(),
    }
}

pub fn setup_builtins() -> Env {
    let env = Env::new();

    // Define a list of all builtin operator names
    let builtins = [
        "+", "-", "*", "/", "%", "**", "^", "&", "|", "head", "last", "tail", "list", "join",
        "range", "eval", "if", "print", "load", "read", "==", "!=", ">", ">=", "<", "<=", "=",
        "def", "\\", "chars", "int", "sort", "len", "str-sub", "split", "and", "or", "not", "<<",
        ">>", "sqrt", "abs", "min", "max", "floor", "ceil", "round", "sin", "cos", "tan", "log",
        "exp", "truncate",
    ];

    for op in builtins {
        env.insert(op.to_string(), Expr::Builtin(op.to_string()));
    }

    env
}
