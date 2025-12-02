use jlisp::ast::Expr;
mod common;
use common::eval_str;

#[test]
fn test_add() {
    let res = eval_str("(+ 1 2 3)").unwrap();
    assert_eq!(res, Expr::Number(6));
}

#[test]
fn test_sub() {
    let res = eval_str("(- 1 2 3)").unwrap();
    assert_eq!(res, Expr::Number(-4));
}

#[test]
fn negate() {
    let res = eval_str("(- 1)").unwrap();
    assert_eq!(res, Expr::Number(-1));
}

#[test]
fn test_mul() {
    let res = eval_str("(* 1 2 3)").unwrap();
    assert_eq!(res, Expr::Number(6));
}

#[test]
fn test_div() {
    let res = eval_str("(/ 1 2)").unwrap();
    assert_eq!(res, Expr::Float(0.5));
}

#[test]
fn test_mod() {
    let res = eval_str("(% 14 10)").unwrap();
    assert_eq!(res, Expr::Number(4));
}
