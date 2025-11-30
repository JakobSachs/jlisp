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

/// Macro to generate builtin function cases
macro_rules! builtin_cases {
    ($sym:expr, $args:expr, $line:expr, $($name:expr => $handler:expr),*) => {
        match $sym {
            $($name => $handler($sym, $args, $line),)*
            _ => panic!(),
        }
    };
}

/// Macro to generate builtin list
macro_rules! builtin_list {
    ($($name:expr),*) => {
        [$($name),*]
    };
}

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
    // Handle arithmetic operators
    if matches!(
        sym,
        "+" | "-" | "*" | "/" | "%" | "**" | "&" | "|" | "^" | "<<" | ">>"
    ) {
        return builtin_op(sym, args, line);
    }

    // Handle comparison operators
    if matches!(sym, "==" | "!=") {
        return builtin_comp(sym, args, line);
    }

    // Handle ordering operators
    if matches!(sym, ">" | "<" | ">=" | "<=") {
        return builtin_ord(sym, args, line);
    }

    // Handle logic operators
    if matches!(sym, "and" | "or" | "not") {
        return builtin_logic(sym, args, line);
    }

    // Handle builtin functions
    builtin_cases!(sym, args, line,
        "head" => builtin_head,
        "last" => builtin_last,
        "tail" => builtin_tail,
        "list" => builtin_list,
        "join" => builtin_join,
        "eval" => |s, a, l| builtin_eval(s, env, a, l),
        "def" => |_s, a, l| builtin_var(env, "def", a, l),
        "=" => |_s, a, l| builtin_var(env, "=", a, l),
        "\\" => |s, a, l| builtin_lambda(s, env, a, l),
        "print" => builtin_print,
        "range" => builtin_range,
        "if" => |s, a, l| builtin_if(s, env, a, l),
        "load" => |s, a, l| builtin_load(s, env, a, l),
        "read" => builtin_read,
        "chars" => builtin_chars,
        "int" => builtin_int,
        "sort" => builtin_sort,
        "len" => builtin_len,
        "str-sub" => builtin_str_sub,
        "split" => builtin_split,
        "sqrt" => builtin_sqrt,
        "abs" => builtin_abs,
        "min" => builtin_min,
        "max" => builtin_max,
        "floor" => builtin_floor,
        "ceil" => builtin_ceil,
        "round" => builtin_round,
        "sin" => builtin_sin,
        "cos" => builtin_cos,
        "tan" => builtin_tan,
        "log" => builtin_log,
        "exp" => builtin_exp,
        "truncate" => builtin_truncate
    )
}

pub fn setup_builtins() -> Env {
    let env = Env::new();

    // Define all builtin names using the macro
    let builtins = builtin_list![
        "+", "-", "*", "/", "%", "**", "^", "&", "|", "<<", ">>",
        "==", "!=", ">", ">=", "<", "<=",
        "and", "or", "not",
        "head", "last", "tail", "list", "join",
        "range", "eval", "if", "print", "load", "read",
        "=", "def", "\\", "chars", "int", "sort", "len", "str-sub", "split",
        "sqrt", "abs", "min", "max", "floor", "ceil", "round",
        "sin", "cos", "tan", "log", "exp", "truncate"
    ];

    for op in builtins {
        env.insert(op.to_string(), Expr::Builtin(op.to_string()));
    }

    env
}
